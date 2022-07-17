use http::{Request, Response};
use unit_rs::{HttpMiddleware, Unit};

fn main() {
    let mut unit = Unit::new().unwrap();

    unit.set_request_handler(HttpMiddleware::new(|req: Request<Vec<u8>>| {
        let body = format!("Hello world!\n\nBody length: {}\n", req.body().len());

        let response = Response::builder()
            .header("Content-Type", "text/plain")
            .body(body.into_bytes())
            .unwrap();

        Ok(response)
    }));

    unit.run();
}
