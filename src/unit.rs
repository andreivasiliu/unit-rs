use std::sync::{Condvar, Mutex, MutexGuard, Once};

use libc::c_void;

use crate::error::{UnitError, UnitInitError, UnitResult};
use crate::nxt_unit::{
    self, nxt_unit_ctx_t, nxt_unit_done, nxt_unit_init, nxt_unit_init_t, nxt_unit_request_done,
    nxt_unit_request_info_t, nxt_unit_response_init, nxt_unit_run,
};
use crate::request::UnitRequest;

unsafe extern "C" fn app_request_handler(req: *mut nxt_unit_request_info_t) {
    // SAFETY: The context data is passed as Unit context-specific user data,
    // and individual Unit contexts correspond to individual threads.
    let context_data = (*(*req).ctx).data as *mut ContextData;
    let context_data = &mut *context_data;

    let rc = nxt_unit_response_init(req, 200, 1, 0 as u32);

    if rc != nxt_unit::NXT_UNIT_OK as i32 {
        nxt_unit_request_done(req, rc);
        return;
    }

    let rc = if let Some(request_handler) = &mut context_data.request_handler {
        let unit_request = UnitRequest {
            nxt_request: &mut *req,
            _lifetime: Default::default(),
        };
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

struct ContextData {
    request_handler: Option<Box<dyn FnMut(UnitRequest) -> UnitResult<()>>>,
    is_main_context: bool,
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

fn main_context() -> MutexGuard<'static, MainContext> {
    unsafe {
        MAIN_CONTEXT_INIT.call_once(|| {
            MAIN_CONTEXT = Some(Mutex::new(MainContext::new()));
        });
        MAIN_CONTEXT
            .as_ref()
            .expect("Initialized above")
            .lock()
            .expect("Main context should not be poisoned")
    }
}

static mut MAIN_CONTEXT_NOTIFIER: Option<Condvar> = None;
static MAIN_CONTEXT_NOTIFIER_INIT: Once = Once::new();

fn main_context_notifier() -> &'static Condvar {
    unsafe {
        MAIN_CONTEXT_NOTIFIER_INIT.call_once(|| {
            MAIN_CONTEXT_NOTIFIER = Some(Condvar::new());
        });
        MAIN_CONTEXT_NOTIFIER.as_ref().expect("Initialized above")
    }
}

struct MainContext {
    main_unit_context: *mut nxt_unit_ctx_t,
    init_error: Option<UnitInitError>,
    secondary_context_count: usize,
    finalized: bool,
}

impl MainContext {
    fn new() -> Self {
        MainContext {
            main_unit_context: std::ptr::null_mut(),
            init_error: None,
            secondary_context_count: 0,
            finalized: false,
        }
    }
}

/// The Unit application context.
///
/// This object wraps the `libunit` library, which talks to the Unit server over
/// shared memory and a unix socket in order to receive data about requests.
pub struct Unit {
    ctx: *mut nxt_unit_ctx_t,
    context_data: *mut ContextData,
    noop_context: bool,
}

impl Unit {
    /// Create a new Unit context and initialize the Unit application.
    ///
    /// Note: Only one Unit object may be active in a single process.
    pub fn new() -> Result<Self, UnitInitError> {
        let mut main_context = main_context();

        if let Some(error) = main_context.init_error {
            return Err(error);
        }

        if main_context.finalized {
            // The main thread already exited; fast-track all future threads to
            // exit as well.
            return Ok(Self {
                ctx: std::ptr::null_mut(),
                context_data: std::ptr::null_mut(),
                noop_context: true,
            });
        }

        if main_context.main_unit_context.is_null() {
            // First context ever created
            let context_data = Box::new(ContextData {
                request_handler: None,
                is_main_context: true,
                unit_is_ready: false,
            });

            let context_user_data = Box::into_raw(context_data);

            let ctx = unsafe {
                let mut init: nxt_unit_init_t = std::mem::zeroed();
                init.callbacks.request_handler = Some(app_request_handler);
                init.callbacks.ready_handler = Some(ready_handler);

                init.ctx_data = context_user_data as *mut c_void;

                nxt_unit_init(&mut init)
            };

            if ctx.is_null() {
                main_context.init_error = Some(UnitInitError);
                return Err(UnitInitError);
            }

            // Run once for the ready handler to be called.
            loop {
                let rc = unsafe { nxt_unit::nxt_unit_run_once(ctx) };

                if rc != nxt_unit::NXT_UNIT_OK as i32 {
                    main_context.init_error = Some(UnitInitError);
                    return Err(UnitInitError);
                }

                // Check if the ready handler was called.
                unsafe {
                    // SAFETY: This data is thread-specific, and not shared
                    // anywhere.
                    let context_data = (*ctx).data as *mut ContextData;
                    let context_data = &mut *context_data;

                    if context_data.unit_is_ready {
                        break;
                    }
                }
            }

            main_context.main_unit_context = ctx;

            Ok(Self {
                ctx,
                context_data: context_user_data,
                noop_context: false,
            })
        } else {
            // Additional contexts are created from the first
            let context_data = Box::new(ContextData {
                request_handler: None,
                is_main_context: false,
                unit_is_ready: false,
            });

            let context_user_data = Box::into_raw(context_data);

            let ctx = unsafe {
                nxt_unit::nxt_unit_ctx_alloc(
                    main_context.main_unit_context,
                    context_user_data as *mut c_void,
                )
            };

            if ctx.is_null() {
                return Err(UnitInitError);
            }

            main_context.secondary_context_count += 1;

            Ok(Self {
                ctx,
                context_data: context_user_data,
                noop_context: false,
            })
        }
    }

    fn context(&self) -> &ContextData {
        // SAFETY: The only other thing that can access this is `.run()`, which
        // requires `&mut self` and therefore guaranteed not to be active.
        unsafe { &*self.context_data }
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
    pub fn set_request_handler(&mut self, f: impl FnMut(UnitRequest) -> UnitResult<()> + 'static) {
        if self.noop_context {
            return;
        }
        self.context_mut().request_handler = Some(Box::new(f))
    }

    /// Enter the main event loop, handling requests until the Unit server exits
    /// or requests a restart.
    pub fn run(&mut self) {
        if self.noop_context {
            return;
        }

        // SAFETY: Call via FFI into Unit's main loop. It will call back into
        // Rust code using callbacks, which must use catch_unwind to be
        // FFI-safe.
        unsafe {
            nxt_unit_run(self.ctx);
        }
    }
}

// An implementation of drop that waits for all secondary Unit contexts to be
// dropped first in other threads before dropping the main thread context.
impl Drop for Unit {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: This structure is the only owner of the box, and is being
            // dropped, therefore not currently being shared.
            drop(Box::from_raw(self.context_data));
        }

        if !self.context().is_main_context {
            // Secondary context. Drop immediately, but also notify the main
            // context in case it's waiting.
            unsafe {
                nxt_unit_done(self.ctx);
            }
            let mut main_context = main_context();
            main_context.secondary_context_count -= 1;
            drop(main_context);
            let main_context_notifier = main_context_notifier();
            main_context_notifier.notify_all();
        } else {
            // Main context. Wait until all secondary contexts dropped before
            // dropping this one.

            let main_context = main_context();

            if main_context.secondary_context_count != 0 && std::thread::panicking() {
                // Keep the Unit context alive, other threads might be using it.
                // At the same time, don't wait for them, this panic needs to be
                // shown immediately.
                return;
            }

            let notifier_condvar = main_context_notifier();

            // Temporarily release the mutex and wait until all secondary
            // threads finish before destroying the main context.
            let result = notifier_condvar.wait_while(main_context, |main_context| {
                main_context.secondary_context_count != 0
            });

            // If the mutex became poisoned, best course of action is to leak
            // and not touch anything else.
            let mut main_context = match result {
                Ok(main_context) => main_context,
                Err(_) => return,
            };

            main_context.finalized = true;
            assert_eq!(main_context.secondary_context_count, 0);

            unsafe {
                nxt_unit_done(self.ctx);
            }
        }
    }
}
