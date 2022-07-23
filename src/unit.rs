use std::ptr::NonNull;
use std::sync::{Arc, Mutex, MutexGuard, Once, Weak};

use libc::c_void;

use crate::error::{UnitError, UnitInitError, UnitResult};
use crate::nxt_unit::{
    self, nxt_unit_ctx_t, nxt_unit_done, nxt_unit_init, nxt_unit_init_t, nxt_unit_request_done,
    nxt_unit_request_info_t, nxt_unit_response_init, nxt_unit_run,
};
use crate::request::UnitRequest;

unsafe extern "C" fn request_handler(req: *mut nxt_unit_request_info_t) {
    // SAFETY: The context data is passed as Unit context-specific user data,
    // and individual Unit contexts correspond to individual threads.
    let context_data = (*(*req).ctx).data as *mut ContextData;
    let context_data = &mut *context_data;

    let rc = nxt_unit_response_init(req, 200, 1, 0 as u32);

    if rc != nxt_unit::NXT_UNIT_OK as i32 {
        nxt_unit_request_done(req, rc);
        return;
    }

    let rc = if let Some(service) = &mut context_data.request_handler {
        let unit_request = UnitRequest {
            nxt_request: &mut *req,
            _lifetime: Default::default(),
        };
        // FIXME: Wrap in catch_unwind
        match service.handle_request(unit_request) {
            Ok(()) => nxt_unit::NXT_UNIT_OK as i32,
            Err(UnitError(rc)) => rc,
        }
    } else {
        nxt_unit::NXT_UNIT_OK as i32
    };

    nxt_unit_request_done(req, rc);
}

struct ContextData {
    request_handler: Option<Box<dyn UnitService>>,
    unit_is_ready: bool,
}

unsafe extern "C" fn ready_handler(ctx: *mut nxt_unit_ctx_t) -> i32 {
    // SAFETY: This is only ever called once, in the main thread, while no other
    // main thread handlers are running.
    let context_data = (*ctx).data as *mut ContextData;
    let context_data = &mut *context_data;

    context_data.unit_is_ready = true;

    nxt_unit::NXT_UNIT_OK as i32
}

static mut MAIN_CONTEXT: Option<Mutex<MainContext>> = None;
static MAIN_CONTEXT_INIT: Once = Once::new();

enum MainContext {
    Uninitialized,
    InitFailed(UnitInitError),
    Initialized(Weak<UnitContextWrapper>),
}

fn main_context() -> MutexGuard<'static, MainContext> {
    unsafe {
        MAIN_CONTEXT_INIT.call_once(|| {
            MAIN_CONTEXT = Some(Mutex::new(MainContext::Uninitialized));
        });
        MAIN_CONTEXT
            .as_ref()
            .expect("Initialized above")
            .lock()
            .expect("Main context should not be poisoned")
    }
}

/// The Unit application context.
///
/// This object wraps the `libunit` library, which talks to the Unit server over
/// shared memory and a unix socket in order to receive data about requests.
pub struct Unit {
    context_wrapper: Option<Arc<UnitContextWrapper>>,
    context_data: *mut ContextData,
}

impl Unit {
    /// Create a new Unit context and initialize the Unit application.
    ///
    /// Note: Only one Unit object may be active in a single process.
    pub fn new() -> Result<Self, UnitInitError> {
        let mut main_context = main_context();

        let main_unit_context = match &*main_context {
            MainContext::InitFailed(UnitInitError) => {
                return Err(UnitInitError);
            }
            MainContext::Uninitialized => None,
            MainContext::Initialized(main_unit_context) => {
                match main_unit_context.upgrade() {
                    Some(context) => Some(context),
                    None => {
                        // The main thread already exited; fast-track all future threads to
                        // exit as well.
                        return Ok(Self {
                            context_wrapper: None,
                            context_data: std::ptr::null_mut(),
                        });
                    }
                }
            }
        };

        if let Some(main_unit_context) = main_unit_context {
            // Additional contexts are created from the first.

            let context_data = Box::new(ContextData {
                request_handler: None,
                unit_is_ready: false,
            });

            let context_user_data = Box::into_raw(context_data);

            let ctx = unsafe {
                nxt_unit::nxt_unit_ctx_alloc(
                    main_unit_context.context.as_ptr(),
                    context_user_data as *mut c_void,
                )
            };

            let ctx = match NonNull::new(ctx) {
                Some(ctx) => ctx,
                None => {
                    return Err(UnitInitError);
                }
            };

            let context_wrapper = UnitContextWrapper {
                parent_context: Some(main_unit_context.clone()),
                context: ctx,
            };

            Ok(Self {
                context_wrapper: Some(Arc::new(context_wrapper)),
                context_data: context_user_data,
            })
        } else {
            // First context ever created.

            let context_data = Box::new(ContextData {
                request_handler: None,
                unit_is_ready: false,
            });

            let context_user_data = Box::into_raw(context_data);

            let ctx = unsafe {
                let mut init: nxt_unit_init_t = std::mem::zeroed();
                init.callbacks.request_handler = Some(request_handler);
                init.callbacks.ready_handler = Some(ready_handler);

                init.ctx_data = context_user_data as *mut c_void;

                nxt_unit_init(&mut init)
            };

            let ctx = match NonNull::new(ctx) {
                Some(ctx) => ctx,
                None => {
                    *main_context = MainContext::InitFailed(UnitInitError);
                    return Err(UnitInitError);
                }
            };

            // Run until the ready handler is called.
            loop {
                let rc = unsafe { nxt_unit::nxt_unit_run_once(ctx.as_ptr()) };

                if rc != nxt_unit::NXT_UNIT_OK as i32 {
                    *main_context = MainContext::InitFailed(UnitInitError);
                    return Err(UnitInitError);
                }

                // Check if the ready handler was called.
                // SAFETY: This data is thread-specific, and not shared
                // anywhere.
                unsafe {
                    let context_data = (*ctx.as_ptr()).data as *mut ContextData;
                    let context_data = &mut *context_data;

                    if context_data.unit_is_ready {
                        break;
                    }
                }
            }

            let context_wrapper = Arc::new(UnitContextWrapper {
                parent_context: None,
                context: ctx,
            });

            // Keep a global weak reference to this, other Unit contexts will be
            // spawned from it.
            *main_context = MainContext::Initialized(Arc::downgrade(&context_wrapper));

            Ok(Self {
                context_wrapper: Some(context_wrapper),
                context_data: context_user_data,
            })
        }
    }

    fn context_mut(&mut self) -> &mut ContextData {
        // SAFETY: The only other thing that can access this is `.run()`, which
        // requires `&mut self` and therefore guaranteed not to be active.
        unsafe { &mut *self.context_data }
    }

    /// Set a request handler for the Unit application.
    ///
    /// The handler must be a function or lambda function that takes a
    /// [`UnitRequest`](UnitRequest) object and returns a
    /// [`UnitResult<()>`](UnitResult).
    pub fn set_request_handler(&mut self, f: impl UnitService + 'static) {
        if self.context_wrapper.is_none() {
            return;
        }
        self.context_mut().request_handler = Some(Box::new(f))
    }

    /// Enter the main event loop, handling requests until the Unit server exits
    /// or requests a restart.
    pub fn run(&mut self) {
        if let Some(context_wrapper) = &self.context_wrapper {
            // SAFETY: Call via FFI into Unit's main loop. It will call back into
            // Rust code using callbacks, which must use catch_unwind to be
            // FFI-safe.
            unsafe {
                nxt_unit_run(context_wrapper.context.as_ptr());
            }
        }
    }
}

// A wrapper over Unit's context that deallocates the context when dropped.
struct UnitContextWrapper {
    parent_context: Option<Arc<UnitContextWrapper>>,
    context: NonNull<nxt_unit_ctx_t>,
}

impl Drop for UnitContextWrapper {
    fn drop(&mut self) {
        // The order here is important. Secondary contexts are created from a
        // main context, which must be dropped last.

        // SAFETY: This structure is only ever held in an Arc, meaning that this
        // is the last instance of it, and it's being dropped.
        unsafe {
            nxt_unit_done(self.context.as_ptr());
        }

        // This is an Arc, which may or may not call the parent's drop.
        drop(self.parent_context.take());
    }
}

impl Drop for Unit {
    fn drop(&mut self) {
        // SAFETY: This structure is the only owner of the box, and is being
        // dropped, therefore not currently being shared.
        unsafe {
            drop(Box::from_raw(self.context_data));
        }

        // Note: Everything that uses the contex must be dropped before this.
        drop(self.context_wrapper.take());
    }
}

pub trait UnitService {
    fn handle_request(&mut self, req: UnitRequest) -> UnitResult<()>;
}

impl<F> UnitService for F
where
    F: FnMut(UnitRequest) -> UnitResult<()> + 'static,
{
    fn handle_request(&mut self, req: UnitRequest) -> UnitResult<()> {
        self(req)
    }
}
