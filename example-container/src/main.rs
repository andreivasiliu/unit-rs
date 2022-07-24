use unit_rs::{Unit, Request};

fn main() {
    let mut unit = Unit::new().unwrap();

    unit.set_request_handler(|req: Request<'_>| {
        let headers = &[("Content-Type", "text/plain")];
        let body = "Hello world!\n";
        req.create_response(headers, body)?;
    
        Ok(())
    });

    unit.run();
}
