mod nxt_unit;
mod unit;

use nxt_unit::{nxt_unit_init_t, nxt_unit_init, nxt_unit_run, nxt_unit_done, nxt_unit_response_init, nxt_unit_request_info_t, nxt_unit_request_done, nxt_unit_response_add_field, nxt_unit_response_add_content, nxt_unit_response_send, nxt_unit_response_buf_alloc, nxt_unit_buf_t, nxt_unit_request_read, size_t, nxt_unit_buf_send};

fn main() {
    let threads = 4;

    unsafe { uxt_main(threads) };
}

unsafe fn uxt_main(_threads: i32) {
    let mut init: nxt_unit_init_t = std::mem::zeroed();
    init.callbacks.request_handler = Some(greeting_app_request_handler);

    let ctx = nxt_unit_init(&mut init);
    if ctx.is_null() {
        std::process::exit(1);
    }
    nxt_unit_run(ctx);

    nxt_unit_done(ctx);
}

static RETURNCONTENT: &str = "Rust: Hello world!\n";
static CONTENTTYPE: &str = "Content-Type";
static TEXTPLAIN: &str = "text/plain";

unsafe extern "C" fn greeting_app_request_handler(req: *mut nxt_unit_request_info_t) {
    let mut rc = nxt_unit_response_init(
        req,
        200 as u16,
        1 as u32,
        (CONTENTTYPE.len() + TEXTPLAIN.len() + RETURNCONTENT.len()) as u32,
    );

    if rc != nxt_unit::NXT_UNIT_OK as i32 {
        nxt_unit_request_done(req, rc);
        return;
    }

    rc = nxt_unit_response_add_field(
        req,
        CONTENTTYPE.as_ptr() as *const libc::c_char,
        CONTENTTYPE.len() as u8,
        TEXTPLAIN.as_ptr() as *const libc::c_char,
        TEXTPLAIN.len() as u32,
    );

    if rc != nxt_unit::NXT_UNIT_OK as i32 {
        nxt_unit_request_done(req, rc);
        return;
    }

    rc = nxt_unit_response_add_content(
        req,
        RETURNCONTENT.as_ptr() as *const libc::c_void,
        RETURNCONTENT.len() as u32,
    );

    if rc != nxt_unit::NXT_UNIT_OK as i32 {
        nxt_unit_request_done(req, rc);
        return;
    }

    rc = nxt_unit_response_send(req);

    if rc != nxt_unit::NXT_UNIT_OK as i32 {
        nxt_unit_request_done(req, rc);
        return;
    }

    let buf = nxt_unit_response_buf_alloc(
        req,
        (*(*req).request_buf)
            .end
            .offset_from((*(*req).request_buf).start) as u32,
    );

    if buf == 0 as *mut libc::c_void as *mut nxt_unit_buf_t {
        nxt_unit_request_done(req, rc);
        return;
    }

    let res = nxt_unit_request_read(
        req,
        (*buf).free as *mut libc::c_void,
        (*buf).end.offset_from((*buf).free) as libc::c_long as size_t,
    );
    (*buf).free = (*buf).free.offset(res as isize);

    rc = nxt_unit_buf_send(buf);
    nxt_unit_request_done(req, rc);
}
