// An example inspired by the official example application written in C:
// https://github.com/nginx/unit/blob/bba97134e9/src/test/nxt_unit_app_test.c

use std::io::Write;

use unit_rs::{Unit, UnitRequest, UnitResult};

fn main() {
    let mut unit = Unit::new().unwrap();

    unit.set_request_handler(request_handler);

    unit.run();
}

fn request_handler(req: UnitRequest) -> UnitResult<()> {
    // Create and send a response.
    let headers = &[("Content-Type", "text/plain")];
    let mut res = req.create_response(headers, "Hello world!\n")?;

    // NGINX Unit uses "Transfer-Encoding: chunked" by default, and can send
    // additional chunks after the initial response was already sent to the
    // client.
    res.send_buffer_with_writer(4096, |req, w| {
        write!(w, "Request data:\n")?;
        write!(w, "  Method: {}\n", req.method())?;
        write!(w, "  Protocol: {}\n", req.version())?;
        write!(w, "  Remote addr: {}\n", req.remote())?;
        write!(w, "  Local addr: {}\n", req.local())?;
        write!(w, "  Server name: {}\n", req.server_name())?;
        write!(w, "  Target: {}\n", req.target())?;
        write!(w, "  Path: {}\n", req.path())?;
        write!(w, "  Query: {}\n", req.query())?;
        write!(w, "  Fields:\n")?;
        for (name, value) in req.fields() {
            write!(w, "    {}: {}\n", name, value).unwrap();
        }
        write!(w, "  Body:\n    ").unwrap();

        w.copy_from_reader(req.body())?;

        Ok(())
    })
    .unwrap(); // FIXME

    Ok(())
}
