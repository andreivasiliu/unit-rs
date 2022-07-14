use libc::c_void;

use crate::nxt_unit::{
    nxt_unit_request_info_t, nxt_unit_request_read, nxt_unit_sptr_get, nxt_unit_sptr_t,
};
use crate::response::{add_response, UnitResponse};
use crate::unit::UnitResult;

pub struct UnitRequest {
    pub(crate) nxt_request: *mut nxt_unit_request_info_t,
}

impl UnitRequest {
    pub fn create_response<'a>(
        self,
        headers: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
        body: impl AsRef<[u8]>,
    ) -> UnitResult<UnitResponse> {
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

    pub fn method(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.method, r.method_length.into())
        }
    }

    pub fn version(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.version, r.version_length.into())
        }
    }

    pub fn remote(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.remote, r.remote_length.into())
        }
    }

    pub fn local(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.local, r.local_length.into())
        }
    }

    pub fn server_name(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.server_name, r.server_name_length)
        }
    }

    pub fn target(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.target, r.target_length)
        }
    }

    pub fn path(&self) -> &str {
        unsafe {
            let r = &(*(*self.nxt_request).request);
            sptr_to_slice(&r.path, r.path_length)
        }
    }

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
    std::str::from_utf8_unchecked(slice)
}
