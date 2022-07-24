// An example that uses the HttpHandler, an adapter that converts Unit's Request
// and Response objects into types from the `http` crate.

use http::{Request, Response};
use unit_rs::{http::HttpHandler, Unit};

fn main() {
    let mut unit = Unit::new().unwrap();

    unit.set_request_handler(HttpHandler::new(|req: Request<Vec<u8>>| {
        let body = format!("Hello world!\n\nBody length: {}\n", req.body().len());

        // The Unit server redirects stdout/stderr to the server log.
        eprintln!("Received reqest for: {}", req.uri().path());

        if req.uri().path() == "/panic" {
            // The HttpHandler converts panics into 501 errors. This imposes an
            // UnwindSafe restriction on the request handler.
            panic!("The /panic path panics!")
        }

        let response = Response::builder()
            .header("Content-Type", "text/plain")
            .body(body.into_bytes())
            .unwrap();

        Ok(response)
    }));

    unit.run();
}
