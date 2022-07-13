use std::io::Write;

use unit::{Unit, UnitRequest, UnitResult};

mod nxt_unit;
mod unit;

fn main() {
    let mut unit = Unit::new();

    unit.add_request_handler(request_handler);

    unit.run();
}

fn request_handler(req: UnitRequest) -> UnitResult<()> {
    let headers = &[
        ("Content-Type", "text/plain"),
    ];
    let mut res = req.create_response(headers, "Hello world?\n")?;

    res.send_buffer(256, |buf| {
        write!(buf, "Here's a nice buffer!\n").unwrap();
        Ok(())
    })?;

    Ok(())
}