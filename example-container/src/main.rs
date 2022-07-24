use unit_rs::{Unit, Request};

fn main() {
    let mut unit = Unit::new().unwrap();

    unit.set_request_handler(|req: Request<'_>| {
        let headers = &[("Content-Type", "text/plain")];
        let body = "Hello world!\n";
        req.send_response(200, headers, body)?;
    
        Ok(())
    });

    unit.run();
}
