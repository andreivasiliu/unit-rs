use libc::c_void;

use crate::nxt_unit::{
    self, nxt_unit_ctx_t, nxt_unit_done, nxt_unit_init, nxt_unit_init_t, nxt_unit_request_done,
    nxt_unit_request_info_t, nxt_unit_response_init, nxt_unit_run,
};
use crate::request::UnitRequest;

unsafe extern "C" fn app_request_handler(req: *mut nxt_unit_request_info_t) {
    let context_data = (*(*req).ctx).data as *const ContextData;
    let context_data = &*context_data;

    let rc = nxt_unit_response_init(req, 200, 1, 0 as u32);

    if rc != nxt_unit::NXT_UNIT_OK as i32 {
        nxt_unit_request_done(req, rc);
        return;
    }

    let rc = if let Some(request_handler) = &context_data.request_handler {
        let unit_request = UnitRequest { nxt_request: &mut *req, _lifetime: Default::default() };
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

/// The Unit application context.
/// 
/// This object wraps the `libunit` library, which talks to the Unit server over
/// shared memory and a unix socket in order to receive data about requests.
pub struct Unit {
    ctx: *mut nxt_unit_ctx_t,
    context_data: *mut ContextData,
}

struct ContextData {
    request_handler: Option<Box<dyn Fn(UnitRequest) -> UnitResult<()>>>,
}

impl Unit {
    /// Create a new Unit context and initialize the Unit application.
    /// 
    /// Note: Only one Unit object may be active in a single process.
    pub fn new() -> Self {
        let context_data = Box::new(ContextData {
            // FIXME: This might drop requests.
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

    /// Set a request handler for the Unit application.
    /// 
    /// The handler must be a function or lambda function that takes a
    /// [`UnitRequest`](UnitRequest) object and returns a
    /// [`UnitResult<()>`](UnitResult).
    pub fn set_request_handler(&mut self, f: impl Fn(UnitRequest) -> UnitResult<()> + 'static) {
        self.context_mut().request_handler = Some(Box::new(f))
    }

    /// Enter the main event loop, handling requests until the Unit server exits
    /// or requests a restart.
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

/// Error code returned by the Unit library.
pub struct UnitError(pub(crate) i32);

/// Result type returned from methods that have a [`UnitError`](UnitError)
/// error.
pub type UnitResult<T> = Result<T, UnitError>;

pub(crate) trait IntoUnitResult {
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
