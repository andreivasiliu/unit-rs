use std::io::Write;

use unit::{Unit, UnitRequest, UnitResult};

mod nxt_unit;
mod unit;

fn main() {
    let mut unit = Unit::new();

    unit.set_request_handler(request_handler);

    unit.run();
}

fn request_handler(req: UnitRequest) -> UnitResult<()> {
    // Create and send a response.
    let headers = &[("Content-Type", "text/plain")];
    let mut res = req.create_response(headers, "Hello world!\n")?;

    // Nginx Unit uses "Transfer-Encoding: chunked" by default, and can send
    // additional chunks after the initial response was already sent to the
    // client.
    res.send_buffer(256, |buf| {
        write!(buf, "With an additional buffer!\n").unwrap();
        Ok(())
    })?;

    Ok(())
}
