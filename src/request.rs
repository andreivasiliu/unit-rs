use std::io::Read;
use std::marker::PhantomData;

use libc::c_void;

use crate::error::{IntoUnitResult, UnitResult};
use crate::nxt_unit::{self, nxt_unit_request_info_t, nxt_unit_sptr_get, nxt_unit_sptr_t};
use crate::response::Response;
use crate::{BodyWriter, UnitError};

/// A request received by the NGINX Unit server.
///
/// This object can be used to inspect the properties and headers of the
/// request, and send a response back to the client.
pub struct Request<'a> {
    pub(crate) nxt_request: *mut nxt_unit_request_info_t,
    pub(crate) _lifetime: PhantomData<&'a mut ()>,
}

impl<'a> Request<'a> {
    /// Allocate a buffer for the initial response, capable of containing at
    /// most `max_fields_count` fields (headers), and at most
    /// `max_response_size` bytes for the field names, field values, and
    /// response content combined.
    ///
    /// The buffer can be filled and sent using the methods on the returned
    /// [`Response`].
    ///
    /// The buffer can be resized with [`Response::realloc`].
    pub fn create_response(
        &'a self,
        status_code: u16,
        max_fields_count: usize,
        max_response_size: usize,
    ) -> UnitResult<Response<'a>> {
        // SAFETY: Unit's C API will return an error if the response was already
        // sent.
        // This structure is neither Send nor Sync, so parallel responses may
        // exist safely (but may result in error results).
        unsafe {
            nxt_unit::nxt_unit_response_init(
                self.nxt_request,
                status_code,
                max_fields_count as u32,
                max_response_size as u32,
            )
            .into_unit_result()?;
        }

        Ok(Response { request: self })
    }

    /// Send an initial response to the client.
    ///
    /// This is a convenience method that calls the [`Request::create_response`],
    /// [`Response::add_field`], [`Response::add_content`], and
    /// [`Response::send`] methods.
    ///
    /// # Panic
    /// This method will panic if the header name is longer than `u8::MAX`
    /// bytes, or if the header value is longer than `u32::MAX`.
    pub fn send_response(
        &self,
        status_code: u16,
        headers: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
        body: impl AsRef<[u8]>,
    ) -> UnitResult<()> {
        let headers_size: usize = headers
            .iter()
            .map(|(name, value)| name.as_ref().len() + value.as_ref().len())
            .sum();
        let response_size = headers_size + body.as_ref().len();

        for (name, value) in headers {
            assert!(name.as_ref().len() <= u8::MAX as usize);
            assert!(value.as_ref().len() <= u32::MAX as usize);
        }

        let response = self.create_response(status_code, headers.len(), response_size)?;
        for (name, value) in headers {
            response.add_field(name, value)?;
        }
        response.add_content(body)?;
        response.send()
    }

    /// Allocate and send additional response chunks to the client, using a
    /// writer to copy data into the chunks.
    ///
    /// A chunk will be immediately sent to the client once the writer's memory
    /// buffer reaches `chunk_size`, or [`flush()`](std::io::Write::flush) is
    /// called on the writer.
    ///
    /// The writer will also flush when dropped, but may panic in case of errors.
    ///
    /// # Panic
    /// Panics if flushing was not successful when flushing during a drop. It is
    /// recommended to manually call [`flush()`](std::io::Write::flush), or use
    /// the [`Request::send_chunks_with_writer()`] method.
    pub fn write_chunks(&'a self, chunk_size: usize) -> std::io::Result<BodyWriter<'a>> {
        BodyWriter::new(self, chunk_size)
    }

    /// Allocate and send additional response chunks to the client.
    ///
    /// This is similar to [`write_chunks()`](Request::write_chunks), but will
    /// also flush the writer before returning, convert the result into a
    /// [`UnitError`], and log the error.
    ///
    /// This is useful to prevent errors from being silently ignored if the
    /// writer needs to flush while being dropped.
    pub fn send_chunks_with_writer<T>(
        &'a self,
        chunk_size: usize,
        f: impl FnOnce(&mut BodyWriter<'a>) -> std::io::Result<T>,
    ) -> UnitResult<T> {
        let write = || -> std::io::Result<T> {
            let mut writer = self.write_chunks(chunk_size)?;
            let result = f(&mut writer)?;
            std::io::Write::flush(&mut writer)?;
            Ok(result)
        };

        write().map_err(|err| {
            self.log(
                LogLevel::Error,
                &format!("Error writing to response: {}", err),
            );
            UnitError::error()
        })
    }

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
    pub fn send_chunk_with_buffer<T>(
        &self,
        size: usize,
        f: impl FnOnce(&mut &mut [u8]) -> UnitResult<T>,
    ) -> UnitResult<T> {
        let req = self.nxt_request;

        assert!(size <= u32::MAX as usize);

        unsafe {
            let buf = nxt_unit::nxt_unit_response_buf_alloc(req, size as u32);

            if buf.is_null() {
                return Err(UnitError(nxt_unit::NXT_UNIT_ERROR as i32));
            }

            libc::memset((*buf).start as *mut c_void, 0, size);

            let mut buf_contents = std::slice::from_raw_parts_mut((*buf).start as *mut u8, size);
            let result = f(&mut buf_contents)?;

            (*buf).free = (*buf).free.add(size - buf_contents.len());

            nxt_unit::nxt_unit_buf_send(buf).into_unit_result()?;

            Ok(result)
        }
    }

    /// Copy bytes from the request body into the target buffer and return the
    /// number of bytes written.
    ///
    /// If the buffer is smaller than the contents of the body, the contents
    /// will be truncated to the size of the buffer.
    pub fn read_body(&self, target: &mut [u8]) -> usize {
        unsafe {
            let bytes = nxt_unit::nxt_unit_request_read(
                self.nxt_request,
                target.as_mut_ptr() as *mut c_void,
                target.len() as u64,
            );
            bytes as usize
        }
    }

    /// Create a reader that implements the [`Read`](std::io::Read) trait,
    /// which will read from the request body in a blocking manner.
    pub fn body(&self) -> BodyReader<'a> {
        BodyReader {
            _lifetime: Default::default(),
            nxt_request: self.nxt_request,
        }
    }

    /// Create an interator over all header (name, value) tuples.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &str)> {
        unsafe {
            let r = &(*(*self.nxt_request).request);

            (0..r.fields_count as isize).into_iter().map(|i| {
                let field = &*r.fields.as_ptr().offset(i);
                let name = sptr_to_slice(&field.name, field.name_length.into());
                let value = sptr_to_slice(&field.value, field.value_length.into());
                (name, value)
            })
        }
    }

    /// Return whether or not the request was encrypted.
    pub fn tls(&self) -> bool {
        unsafe { (*(*self.nxt_request).request).tls != 0 }
    }

    /// Return the method of the request (e.g. "GET").
    pub fn method(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.method, r.method_length.into())
        }
    }

    /// Return the protocol version of the request (e.g. "HTTP/1.1").
    pub fn version(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.version, r.version_length.into())
        }
    }

    /// Return the remote IP address of the client.
    pub fn remote(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.remote, r.remote_length.into())
        }
    }

    /// Return the local IP address of the server.
    pub fn local(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.local, r.local_length.into())
        }
    }

    /// Return the host name of the server.
    pub fn server_name(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.server_name, r.server_name_length)
        }
    }

    /// Return the combined URI path and query string.
    pub fn target(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.target, r.target_length)
        }
    }

    /// Return the URI path.
    pub fn path(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.path, r.path_length)
        }
    }

    /// Return the URI query string.
    pub fn query(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.query, r.query_length)
        }
    }

    /// Log an error message.
    pub fn log<S: AsRef<str>>(&self, level: LogLevel, message: S) {
        unsafe {
            nxt_unit::nxt_unit_req_log(
                self.nxt_request,
                level as i32,
                "%s\0".as_ptr() as *const i8,
                message.as_ref(),
            )
        }
    }
}

unsafe fn sptr_to_slice(sptr: &nxt_unit_sptr_t, length: u32) -> &str {
    let ptr = nxt_unit_sptr_get(sptr) as *mut u8;
    let slice = std::slice::from_raw_parts(ptr, length as usize);
    // FIXME: temporary, NGINX Unit doesn't guarantee this
    std::str::from_utf8(slice).unwrap()
}

#[repr(u32)]
pub enum LogLevel {
    Alert = nxt_unit::NXT_UNIT_LOG_ALERT,
    Error = nxt_unit::NXT_UNIT_LOG_ERR,
    Warning = nxt_unit::NXT_UNIT_LOG_WARN,
    Notice = nxt_unit::NXT_UNIT_LOG_NOTICE,
    Info = nxt_unit::NXT_UNIT_LOG_INFO,
    Debug = nxt_unit::NXT_UNIT_LOG_DEBUG,
}

/// A reader that reads from the request body.
///
/// This reader is non-blocking, as Unit will buffer the whole request body
/// before running the request handler.
pub struct BodyReader<'a> {
    _lifetime: std::marker::PhantomData<&'a ()>,
    nxt_request: *mut nxt_unit_request_info_t,
}

impl BodyReader<'_> {
    /// Convenience function that allocates and copies the request body data
    /// into a [`Vec<u8>`].
    pub fn read_to_vec(&mut self) -> std::io::Result<Vec<u8>> {
        let mut vec = Vec::new();
        self.read_to_end(&mut vec)?;
        Ok(vec)
    }
}

impl std::panic::UnwindSafe for BodyReader<'_> {}

impl std::io::Read for BodyReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // SAFETY: The target is user-provided and initialized.
        // The BodyReader and UnitRequest are not Sync nor Send, so this is
        // thread-safe.
        // This function does not seem to have any sort of error reporting.
        let bytes = unsafe {
            nxt_unit::nxt_unit_request_read(
                self.nxt_request,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u64,
            )
        };
        Ok(bytes as usize)
    }
}
