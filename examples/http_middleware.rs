use http::{Request, Response};
use unit_rs::{BodyReader, HttpMiddleware, Unit};

fn main() {
    let mut unit = Unit::new().unwrap();

    unit.set_request_handler(HttpMiddleware::new(|mut req: Request<BodyReader>| {
        let body = req.body_mut().read_to_vec().unwrap(); // FIXME
        let body = format!("Hello world!\n\nBody length: {}\n", body.len());

        if req.uri().path() == "/panic" {
            panic!("The /panic path panics!")
        }

        eprintln!("{}", req.uri().path());

        let response = Response::builder()
            .header("Content-Type", "text/plain")
            .body(body.into_bytes())
            .unwrap();

        Ok(response)
    }));

    unit.run();
}
