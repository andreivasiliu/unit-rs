use libc::c_void;

use crate::nxt_unit::{
    self, nxt_unit_buf_send, nxt_unit_ctx_t, nxt_unit_done, nxt_unit_init, nxt_unit_init_t,
    nxt_unit_request_done, nxt_unit_request_info_t, nxt_unit_response_add_content,
    nxt_unit_response_add_field, nxt_unit_response_buf_alloc, nxt_unit_response_init,
    nxt_unit_response_send, nxt_unit_run,
};

pub struct UnitError(i32);
pub type UnitResult<T> = Result<T, UnitError>;

unsafe extern "C" fn app_request_handler(req: *mut nxt_unit_request_info_t) {
    let context_data = (*(*req).ctx).data as *const ContextData;
    let context_data = &*context_data;

    let rc = nxt_unit_response_init(req, 200, 1, 0 as u32);

    if rc != nxt_unit::NXT_UNIT_OK as i32 {
        nxt_unit_request_done(req, rc);
        return;
    }

    let rc = if let Some(request_handler) = &context_data.request_handler {
        let unit_request = UnitRequest { nxt_request: req };
        // FIXME: Wrap in catch_unwind
        match request_handler(unit_request) {
            Ok(()) => nxt_unit::NXT_UNIT_OK as i32,
            Err(UnitError(rc)) => rc,
        }
    } else {
        nxt_unit::NXT_UNIT_OK as i32
    };

    nxt_unit_request_done(req, rc);
}

pub struct Unit {
    ctx: *mut nxt_unit_ctx_t,
    context_data: *mut ContextData,
}

struct ContextData {
    request_handler: Option<Box<dyn Fn(UnitRequest) -> UnitResult<()>>>,
}

pub struct UnitRequest {
    nxt_request: *mut nxt_unit_request_info_t,
}

pub struct UnitResponse {
    request: UnitRequest,
}

impl Unit {
    pub fn new() -> Self {
        let context_data = Box::new(ContextData {
            request_handler: None,
        });

        let context_data = Box::into_raw(context_data);

        let ctx = unsafe {
            let mut init: nxt_unit_init_t = std::mem::zeroed();
            init.callbacks.request_handler = Some(app_request_handler);
            init.ctx_data = context_data as *mut c_void;

            nxt_unit_init(&mut init)
        };

        if ctx.is_null() {
            panic!("Could not initialize Unit context");
        }

        Self { ctx, context_data }
    }

    fn context_mut(&mut self) -> &mut ContextData {
        unsafe { &mut *self.context_data }
    }

    pub fn set_request_handler(&mut self, f: impl Fn(UnitRequest) -> UnitResult<()> + 'static) {
        self.context_mut().request_handler = Some(Box::new(f))
    }

    pub fn run(&mut self) {
        unsafe {
            nxt_unit_run(self.ctx);
        }
    }
}

impl Drop for Unit {
    fn drop(&mut self) {
        unsafe {
            nxt_unit_done(self.ctx);
            drop(Box::from_raw(self.context_data));
        }
    }
}

trait IntoUnitResult {
    fn into_unit_result(self) -> UnitResult<()>;
}

impl IntoUnitResult for i32 {
    fn into_unit_result(self) -> UnitResult<()> {
        if self == 0 {
            Ok(())
        } else {
            Err(UnitError(self))
        }
    }
}

unsafe fn add_response(
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
}

impl std::ops::Deref for UnitResponse {
    type Target = UnitRequest;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl UnitResponse {
    pub fn send_buffer<T>(
        &mut self,
        size: usize,
        f: impl Fn(&mut &mut [u8]) -> UnitResult<T>,
    ) -> UnitResult<T> {
        let req = self.request.nxt_request;

        std::fs::write("/tmp/result.txt", format!("Sending...")).unwrap();

        assert!(size <= u32::MAX as usize);

        unsafe {
            let buf = nxt_unit_response_buf_alloc(req, size as u32);

            if buf.is_null() {
                return Err(UnitError(nxt_unit::NXT_UNIT_ERROR as i32));
            }

            libc::memset((*buf).start as *mut c_void, 0, size);

            let mut buf_contents = std::slice::from_raw_parts_mut((*buf).start as *mut u8, size);
            let result = f(&mut buf_contents)?;

            // nxt_unit_req_log(req, NXT_UNIT_LOG_WARN as i32, b"Senging some extra %d".as_ptr() as *const i8, size - buf_contents.len());

            std::fs::write(
                "/tmp/result.txt",
                format!("Size: {}", size - buf_contents.len()),
            )
            .unwrap();

            (*buf).free = (*buf).free.add(size - buf_contents.len());

            nxt_unit_buf_send(buf).into_unit_result()?;

            Ok(result)
        }
    }
}
