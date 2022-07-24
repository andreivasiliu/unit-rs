use std::io::Write;
use std::panic::UnwindSafe;

use libc::c_void;

use crate::nxt_unit::{
    self, nxt_unit_buf_send, nxt_unit_buf_t, nxt_unit_request_info_t, nxt_unit_response_buf_alloc,
};

use crate::error::{IntoUnitResult, UnitError, UnitResult};
use crate::request::Request;

/// A buffer for constructing an initial response.
///
/// This object is created by calling the
/// [`create_response()`](Request::create_response) method on a [`Request`].
///
/// Dropping this object will _not_ send the response; it must be manually sent
/// with the [`Response::send()`] method.
pub struct Response<'a> {
    pub(crate) request: &'a Request<'a>,
}

impl<'a> Response<'a> {
    pub fn add_field<N: AsRef<[u8]>, V: AsRef<[u8]>>(&self, name: N, value: V) -> UnitResult<()> {
        // SAFETY: Unit's C API does state and buffer size checks internally.
        // This structure is not Send nor Sync, so sharing it is fine.
        unsafe {
            nxt_unit::nxt_unit_response_add_field(
                self.request.nxt_request,
                name.as_ref().as_ptr() as *const libc::c_char,
                name.as_ref().len() as u8,
                value.as_ref().as_ptr() as *const libc::c_char,
                value.as_ref().len() as u32,
            )
            .into_unit_result()
        }
    }

    pub fn add_content<C: AsRef<[u8]>>(&self, content: C) -> UnitResult<()> {
        // SAFETY: Unit's C API does state and buffer size checks internally.
        // This structure is not Send nor Sync, so sharing it is fine.
        unsafe {
            nxt_unit::nxt_unit_response_add_content(
                self.request.nxt_request,
                content.as_ref().as_ptr() as *const c_void,
                content.as_ref().len() as u32,
            )
            .into_unit_result()
        }
    }

    pub fn realloc(&self, max_fields_count: usize, max_fields_size: usize) -> UnitResult<()> {
        // SAFETY: Unit's C API does state and buffer size checks internally.
        // This structure is not Send nor Sync, so sharing it is fine.
        unsafe {
            nxt_unit::nxt_unit_response_realloc(
                self.request.nxt_request,
                max_fields_count as u32,
                max_fields_size as u32,
            )
            .into_unit_result()
        }
    }

    pub fn send(&self) -> UnitResult<()> {
        // SAFETY: Unit's C API does state and buffer size checks internally.
        // This structure is not Send nor Sync, so sharing it is fine.
        unsafe { nxt_unit::nxt_unit_response_send(self.request.nxt_request).into_unit_result() }
    }
}

/// A writer that writes to a Unit shared memory response buffer.
///
/// This object is created using [`Request::write_chunks()`] or
/// [`Request::send_chunks_with_writer()`].
///
/// A chunk will be immediately sent to the client once the writer's memory
/// buffer reaches `chunk_size`, or [`flush()`](std::io::Write::flush) is
/// called on the writer, and a new shared memory buffer will be allocated.
///
/// The writer will also flush when dropped, but any errors that happen during
/// a drop will panic.
pub struct BodyWriter<'a> {
    _lifetime: std::marker::PhantomData<&'a mut ()>,
    nxt_request: *mut nxt_unit_request_info_t,
    response_buffer: *mut nxt_unit_buf_t,
    chunk_cursor: *mut u8,
    chunk_size: usize,
    bytes_remaining: usize,
}

impl UnwindSafe for BodyWriter<'_> {}

impl<'a> BodyWriter<'a> {
    pub(crate) fn new(request: &'a Request<'a>, chunk_size: usize) -> std::io::Result<Self> {
        let mut writer = BodyWriter {
            _lifetime: Default::default(),
            nxt_request: request.nxt_request,
            response_buffer: std::ptr::null_mut(),
            chunk_cursor: std::ptr::null_mut(),
            chunk_size,
            bytes_remaining: 0,
        };
        writer.allocate_buffer()?;
        Ok(writer)
    }

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
                if let Err(err) = self.flush() {
                    // Prevent a double-panic, which causes an abort and hides
                    // details on the initial panic.
                    if !std::thread::panicking() {
                        panic!("Error while dropping ResponseWriter: {}", err);
                    }
                }
            }
        }
    }
}
