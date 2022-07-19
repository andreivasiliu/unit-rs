use std::io::Write;
use std::panic::UnwindSafe;

use libc::c_void;

use crate::nxt_unit::{
    self, nxt_unit_buf_send, nxt_unit_buf_t, nxt_unit_request_info_t,
    nxt_unit_response_add_content, nxt_unit_response_add_field, nxt_unit_response_buf_alloc,
    nxt_unit_response_init, nxt_unit_response_send,
};

use crate::error::{IntoUnitResult, UnitError, UnitResult};
use crate::request::UnitRequest;

/// An object used to send follow-up response bytes to a request, obtained by
/// calling a [`UnitRequest`](UnitRequest)'s
/// [`create_response()`](UnitRequest::create_response) method. Dropping this
/// object will end the response.
pub struct UnitResponse<'a> {
    pub(crate) request: UnitRequest<'a>,
}

impl<'a> std::ops::Deref for UnitResponse<'a> {
    type Target = UnitRequest<'a>;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl<'a> UnitResponse<'a> {
    /// Send another chunk of bytes for this request's response. The bytes will
    /// be immediately sent to the client.
    ///
    /// This method allocates a buffer in Unit's shared memory region, and calls
    /// a user function to fill it.
    ///
    /// The user function receives a `&mut &mut [u8]` slice, and the `write!`
    /// macro can be used to advance the start position of the slice. Only the
    /// bytes between the original start and the new start positions will be
    /// sent, and the rest will be discarded.
    pub fn send_buffer<T>(
        &mut self,
        size: usize,
        f: impl FnOnce(&UnitRequest, &mut &mut [u8]) -> UnitResult<T>,
    ) -> UnitResult<T> {
        let req = self.request.nxt_request;

        assert!(size <= u32::MAX as usize);

        unsafe {
            let buf = nxt_unit_response_buf_alloc(req, size as u32);

            if buf.is_null() {
                return Err(UnitError(nxt_unit::NXT_UNIT_ERROR as i32));
            }

            libc::memset((*buf).start as *mut c_void, 0, size);

            let mut buf_contents = std::slice::from_raw_parts_mut((*buf).start as *mut u8, size);
            let result = f(&self.request, &mut buf_contents)?;

            // nxt_unit_req_log(req, NXT_UNIT_LOG_WARN as i32, b"Senging some extra %d".as_ptr() as *const i8, size - buf_contents.len());

            (*buf).free = (*buf).free.add(size - buf_contents.len());

            nxt_unit_buf_send(buf).into_unit_result()?;

            Ok(result)
        }
    }

    pub fn send_buffer_with_writer<T>(
        &'a mut self,
        chunk_size: usize,
        f: impl FnOnce(&UnitRequest, &mut BodyWriter<'a>) -> std::io::Result<T>,
    ) -> std::io::Result<T> {
        let mut writer = BodyWriter {
            _lifetime: Default::default(),
            nxt_request: self.nxt_request,
            response_buffer: std::ptr::null_mut(),
            chunk_cursor: std::ptr::null_mut(),
            chunk_size,
            bytes_remaining: 0,
        };
        writer.allocate_buffer()?;
        let result = f(&self.request, &mut writer)?;
        writer.flush()?;
        Ok(result)
    }
}

pub(crate) unsafe fn add_response(
    req: *mut nxt_unit_request_info_t,
    headers: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
    body: impl AsRef<[u8]>,
    response_size: usize,
) -> UnitResult<()> {
    nxt_unit_response_init(req, 200, headers.len() as u32, response_size as u32)
        .into_unit_result()?;

    for (header_name, header_value) in headers {
        nxt_unit_response_add_field(
            req,
            header_name.as_ref().as_ptr() as *const libc::c_char,
            header_name.as_ref().len() as u8,
            header_value.as_ref().as_ptr() as *const libc::c_char,
            header_value.as_ref().len() as u32,
        )
        .into_unit_result()?;
    }

    nxt_unit_response_add_content(
        req,
        body.as_ref().as_ptr() as *const libc::c_void,
        body.as_ref().len() as u32,
    )
    .into_unit_result()?;

    nxt_unit_response_send(req).into_unit_result()?;

    Ok(())
}

pub struct BodyWriter<'a> {
    _lifetime: std::marker::PhantomData<&'a mut ()>,
    nxt_request: *mut nxt_unit_request_info_t,
    response_buffer: *mut nxt_unit_buf_t,
    chunk_cursor: *mut u8,
    chunk_size: usize,
    bytes_remaining: usize,
}

impl UnwindSafe for BodyWriter<'_> {}

impl BodyWriter<'_> {
    fn allocate_buffer(&mut self) -> std::io::Result<()> {
        unsafe {
            let buf = nxt_unit_response_buf_alloc(self.nxt_request, self.chunk_size as u32);

            if buf.is_null() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Could not allocate response buffer in Unit's shared memory",
                ));
            }

            self.response_buffer = buf;
            self.chunk_cursor = (*buf).start as *mut u8;
            self.bytes_remaining = self.chunk_size;
        }

        Ok(())
    }

    /// Copy from a reader to this writer without using an intermediary buffer.
    ///
    /// Normally the [`Write`](std::io::Write) trait receives an input buffer to
    /// copy from, and the `ResponseWriter` writer will copy from it into Unit's
    /// shared memory.
    ///
    /// This method will instead give Unit's shared memory buffer directly to
    /// the [`Read`](std::io::Read) trait in order to skip copying to a third
    /// temporary buffer (such as when using [`std::io::copy`]).
    pub fn copy_from_reader<R: std::io::Read>(&mut self, mut r: R) -> std::io::Result<()> {
        loop {
            if self.bytes_remaining == 0 {
                self.flush()?;
                self.allocate_buffer()?;
            }

            // SAFETY: Allocated by Unit and fully initialized with memset.
            // TODO: The memset is unnecessary, use std::io::ReadBuf once that
            // is stabilized.
            let write_buffer = unsafe {
                libc::memset(self.chunk_cursor as *mut c_void, 0, self.bytes_remaining);
                std::slice::from_raw_parts_mut(self.chunk_cursor, self.bytes_remaining)
            };

            let bytes = r.read(write_buffer)?;

            self.chunk_cursor = unsafe { self.chunk_cursor.add(bytes) };
            self.bytes_remaining -= bytes;

            if bytes == 0 {
                break;
            }
        }

        return Ok(());
    }
}

impl std::io::Write for BodyWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buf = if buf.len() >= self.bytes_remaining && !buf.is_empty() {
            if self.bytes_remaining == 0 {
                self.flush()?;
                self.allocate_buffer()?;
            }

            &buf[..buf.len().min(self.bytes_remaining)]
        } else {
            buf
        };

        // SAFETY: The target region is not initialized and never made
        // available to the user until it is, so it cannot overlap.
        // The buffer length is truncated above to fit the target's limit.
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr(), self.chunk_cursor, buf.len());
            self.chunk_cursor = self.chunk_cursor.add(buf.len());
        }
        self.bytes_remaining -= buf.len();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.response_buffer.is_null() || self.bytes_remaining == self.chunk_size {
            return Ok(());
        }

        unsafe {
            (*self.response_buffer).free = (*self.response_buffer)
                .start
                .add(self.chunk_size - self.bytes_remaining);
            nxt_unit_buf_send(self.response_buffer)
                .into_unit_result()
                .map_err(|UnitError(_)| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Could not send response buffer to Unit server",
                    )
                })?;
        }

        self.response_buffer = std::ptr::null_mut();
        self.bytes_remaining = 0;

        Ok(())
    }
}

impl Drop for BodyWriter<'_> {
    fn drop(&mut self) {
        if !self.chunk_cursor.is_null() {
            if std::thread::panicking() {
                unsafe {
                    nxt_unit::nxt_unit_buf_free(self.response_buffer);
                }
            } else {
                // Note: This cannot happen unless there's a bug in unit-rs, as
                // the writer is flushed at the end of send_buffer_with_writer.
                self.flush().expect("Error while dropping ResponseWriter");
            }
        }
    }
}
