use std::io::Read;
use std::marker::PhantomData;

use libc::c_void;

use crate::error::UnitResult;
use crate::nxt_unit::{
    nxt_unit_request_info_t, nxt_unit_request_read, nxt_unit_sptr_get, nxt_unit_sptr_t,
};
use crate::response::{add_response, UnitResponse};

/// A request received by the Nginx Unit server.
///
/// This object can be used to inspect the properties and headers of the
/// request, and send a response back to the client.
pub struct UnitRequest<'a> {
    pub(crate) nxt_request: *mut nxt_unit_request_info_t,
    pub(crate) _lifetime: PhantomData<&'a mut ()>,
}

impl<'a> UnitRequest<'a> {
    /// Send an initial response to the client, and return a
    /// [`UnitResponse`](UnitResponse) object that allows sending additional
    /// data chunks to the client.
    ///
    /// This method will consume the request object and wrap it in a response
    /// object. The returned object will still deref to a reference of a
    /// request object, allowing inspection of the request but no longer
    /// allowing the initiation of a second response.
    pub fn create_response(
        self,
        headers: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
        body: impl AsRef<[u8]>,
    ) -> UnitResult<UnitResponse<'a>> {
        let req = self.nxt_request;

        let headers_size: usize = headers
            .iter()
            .map(|(name, value)| name.as_ref().len() + value.as_ref().len())
            .sum();
        let response_size = headers_size + body.as_ref().len();

        assert!(response_size <= u32::MAX as usize);
        assert!(headers.len() <= u32::MAX as usize);

        for (header_name, header_value) in headers {
            assert!(header_name.as_ref().len() <= u8::MAX as usize);
            assert!(header_value.as_ref().len() <= u32::MAX as usize);
        }

        unsafe {
            add_response(req, headers, body, response_size)?;
        }

        // Consume the object by wrapping it so that this method can never
        // be called again on it.
        // Note that because of Deref, methods that take by reference can
        // still be called.
        Ok(UnitResponse { request: self })
    }

    /// Copy bytes from the request body into the target buffer and return the
    /// number of bytes written.
    ///
    /// If the buffer is smaller than the contents of the body, the contents
    /// will be truncated to the size of the buffer.
    pub fn read_body(&self, target: &mut [u8]) -> usize {
        unsafe {
            let bytes = nxt_unit_request_read(
                self.nxt_request,
                target.as_mut_ptr() as *mut c_void,
                target.len() as u64,
            );
            bytes as usize
        }
    }

    /// Create a reader that implements that [`Read`](std::io::Read) trait,
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
}

unsafe fn sptr_to_slice(sptr: &nxt_unit_sptr_t, length: u32) -> &str {
    let ptr = nxt_unit_sptr_get(sptr) as *mut u8;
    let slice = std::slice::from_raw_parts(ptr, length as usize);
    // FIXME: temporary, Nginx Unit doesn't guarantee this
    std::str::from_utf8(slice).unwrap()
}

pub struct BodyReader<'a> {
    _lifetime: std::marker::PhantomData<&'a ()>,
    nxt_request: *mut nxt_unit_request_info_t,
}

impl BodyReader<'_> {
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
            nxt_unit_request_read(
                self.nxt_request,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u64,
            )
        };
        Ok(bytes as usize)
    }
}
