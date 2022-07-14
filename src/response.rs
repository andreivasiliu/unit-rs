use libc::c_void;

use crate::nxt_unit::{
    self, nxt_unit_buf_send, nxt_unit_request_info_t, nxt_unit_response_add_content,
    nxt_unit_response_add_field, nxt_unit_response_buf_alloc, nxt_unit_response_init,
    nxt_unit_response_send,
};

use crate::request::UnitRequest;
use crate::unit::{IntoUnitResult, UnitError, UnitResult};

pub struct UnitResponse<'a> {
    pub(crate) request: UnitRequest<'a>,
}

impl<'a> std::ops::Deref for UnitResponse<'a> {
    type Target = UnitRequest<'a>;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl UnitResponse<'_> {
    pub fn send_buffer<T>(
        &mut self,
        size: usize,
        f: impl Fn(&UnitRequest, &mut &mut [u8]) -> UnitResult<T>,
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
