//! This module contains an adapter to the Request and Response types from the
//! [`http`](https://docs.rs/http) crate.
//!
//! # Example
//!
//! ```no_run
//! use http::{Request, Response};
//! use unit_rs::{http::HttpHandler, Unit};
//!
//! fn main() {
//!     let mut unit = Unit::new().unwrap();
//!
//!     unit.set_request_handler(HttpHandler::new(|req: Request<Vec<u8>>| {
//!         let body = format!("Hello world!\n\nBody length: {}\n", req.body().len());
//!
//!         let response = Response::builder()
//!             .header("Content-Type", "text/plain")
//!             .body(body.into_bytes())?;
//!
//!         Ok(response)
//!     }));
//!
//!     unit.run();
//! }
//! ```

use std::panic::{AssertUnwindSafe, RefUnwindSafe, UnwindSafe};

use http::{uri::PathAndQuery, Uri};

use crate::{request::LogLevel, unit::UnitService, UnitError, UnitResult};

pub use http::{Request, Response};

/// A trait for request handlers that uses types from the [`http`] crate.
///
/// [`http`]: https://docs.rs/http
///
/// Handlers that implement this trait can be used with the [`HttpHandler`]
/// adapter to convert them to a [`UnitService`].
pub trait HttpService: UnwindSafe {
    fn handle_request(
        &self,
        _req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error>>;
}

/// Adapter to use types from the [`http`] crate.
///
/// [`http`]: https://docs.rs/http
///
/// The inner request handler must implement the [`HttpService`] trait.
pub struct HttpHandler<H: HttpService>(H);

impl<H: HttpService> HttpHandler<H> {
    pub fn new(unit_service: H) -> Self {
        Self(unit_service)
    }
}

impl<H: HttpService + RefUnwindSafe> UnitService for HttpHandler<H> {
    fn handle_request(&mut self, mut req: crate::request::Request<'_>) -> UnitResult<()> {
        self.handle_request_with_http(&mut req).map_err(|err| {
            req.log(LogLevel::Error, err.to_string());
            UnitError::error()
        })
    }
}

impl<H: HttpService + RefUnwindSafe> HttpHandler<H> {
    fn handle_request_with_http(
        &self,
        req: &mut crate::request::Request<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path_and_query: PathAndQuery = req.target().parse()?;
        let uri = Uri::builder()
            .scheme(if req.tls() { "https" } else { "http" })
            .authority(req.server_name())
            .path_and_query(path_and_query)
            .build()?;
        let mut http_request_builder = Request::builder();

        for (name, value) in req.fields() {
            http_request_builder = http_request_builder.header(name, value);
        }

        let http_request = http_request_builder
            .uri(uri)
            .method(req.method())
            .body(req.body().read_to_vec()?)?;

        // SAFETY:
        // The only !UnwindSafe part of http::Request is its Extensions, and
        // this library does not add any extensions to it.
        let http_request = AssertUnwindSafe(http_request);
        let handler = &self.0;

        let http_response = std::panic::catch_unwind(move || {
            let http_request = http_request;
            handler.handle_request(http_request.0)
        });

        match http_response {
            Ok(Ok(http_response)) => {
                let header_count = http_response.headers().len();
                let headers_size: usize = http_response
                    .headers()
                    .iter()
                    .map(|(name, value)| name.as_str().len() + value.as_bytes().len())
                    .sum();
                let body_size = http_response.body().len();

                let response = req.create_response(
                    http_response.status().as_u16(),
                    header_count,
                    headers_size + body_size,
                )?;

                for (name, value) in http_response.headers() {
                    response.add_field(name, value)?;
                }
                response.add_content(http_response.body())?;
                response.send()?;
            }
            Ok(Err(err)) => {
                let content_type = ("Content-Type", "text/plain");
                let response_body = format!("The server experienced an internal error: {}", err);
                let response = req.create_response(
                    501,
                    1,
                    content_type.0.len() + content_type.1.len() + response_body.len(),
                )?;
                response.add_field(content_type.0, content_type.1)?;
                response.add_content(response_body)?;
                response.send()?;
            }
            Err(panic_payload) => {
                req.log(LogLevel::Error, "Panicked during http request handling.");

                // If a request has not been created yet, Unit will generate a
                // 503 error itself, which is what we want.
                std::panic::resume_unwind(panic_payload);
            }
        }
        Ok(())
    }
}

impl<F> HttpService for F
where
    F: Fn(Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error>>,
    F: UnwindSafe + 'static,
{
    fn handle_request(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error>> {
        self(req)
    }
}
