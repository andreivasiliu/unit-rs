// An example inspired by the official example application written in C:
// https://github.com/nginx/unit/blob/bba97134e9/src/test/nxt_unit_app_test.c

use std::io::Write;

use unit_rs::{Unit, UnitRequest, UnitResult};

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
    res.send_buffer(4096, |req, buf| {
        write!(buf, "Request data:\n").unwrap();
        write!(buf, "  Method: {}\n", req.method()).unwrap();
        write!(buf, "  Protocol: {}\n", req.version()).unwrap();
        write!(buf, "  Remote addr: {}\n", req.remote()).unwrap();
        write!(buf, "  Local addr: {}\n", req.local()).unwrap();
        write!(buf, "  Server name: {}\n", req.server_name()).unwrap();
        write!(buf, "  Target: {}\n", req.target()).unwrap();
        write!(buf, "  Path: {}\n", req.path()).unwrap();
        write!(buf, "  Query: {}\n", req.query()).unwrap();
        write!(buf, "  Fields:\n").unwrap();
        for (name, value) in req.fields() {
            write!(buf, "    {}: {}\n", name, value).unwrap();
        }
        write!(buf, "  Body:\n").unwrap();
        let bytes = req.read_body(buf);

        // Advance write pointer with the number of bytes read
        let inner_buf: &mut [u8] = std::mem::take(buf);
        *buf = &mut inner_buf[bytes..];
        write!(buf, "\n  Body bytes: {}\n", bytes).unwrap();
        Ok(())
    })?;

    Ok(())
}
