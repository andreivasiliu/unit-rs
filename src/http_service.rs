use std::panic::{AssertUnwindSafe, UnwindSafe};

use http::{uri::PathAndQuery, Request, Response, Uri};

use crate::{unit::UnitService, BodyReader, UnitRequest, UnitResult};

/// A request handler that uses types from the [`http`] crate.
///
/// [`http`]: https://docs.rs/http
///
/// Request handles with this trait can be used with the [`HttpMiddleware`]
/// adapter.
///
/// ## Example
///
/// ```no_run
/// # use unit_rs::{HttpMiddleware, Unit};
///
/// # fn main() {
/// let mut unit = Unit::new().unwrap();
///
/// unit.set_request_handler(HttpMiddleware::new(|req: http::Request<Vec<u8>>| {
///     let body = format!("Hello world!\n\nBody length: {}\n", req.body().len());
///
///     let response = http::Response::builder()
///         .header("Content-Type", "text/plain")
///         .body(body.into_bytes())
///         .unwrap();
///
///     Ok(response)
/// }));
/// # }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub trait HttpService: UnwindSafe {
    fn handle_request(&mut self, _req: Request<BodyReader>) -> UnitResult<Response<Vec<u8>>>;
}

/// Adapter middleware to use types from the [`http`] crate.
///
/// [`http`]: https://docs.rs/http
///
/// The inner request handler must implement the [`HttpService`] trait.
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub struct HttpMiddleware<H: HttpService>(H);

impl<H: HttpService> HttpMiddleware<H> {
    pub fn new(unit_service: H) -> Self {
        Self(unit_service)
    }
}

impl<H: HttpService> UnitService for HttpMiddleware<H> {
    // TODO: Handle errors, this is currently just a prototype
    fn handle_request(&mut self, req: UnitRequest) -> UnitResult<()> {
        // FIXME: Fix unwrap
        let path_and_query: PathAndQuery = req.target().parse().unwrap();

        // FIXME: Fix unwrap
        // FIXME: Where to get the scheme from?
        let uri = Uri::builder()
            .scheme("http")
            .authority(req.server_name())
            .path_and_query(path_and_query)
            .build()
            .unwrap();

        let mut http_request_builder = Request::builder();

        for (name, value) in req.fields() {
            http_request_builder = http_request_builder.header(name, value);
        }

        let http_request = http_request_builder
            .uri(uri)
            .method(req.method())
            .body(req.body())
            .unwrap();

        // SAFETY:
        // Using AssertUnwindSafe here because the only unsafe object inside
        // this closure is `http::Request`, but it is often thrown away in case
        // of a panic.
        // The HttpHandler trait has the UnwindSafe requirement, and is only
        // implemented for Fn() instead of FnMut() (which would not be
        // UnwindSafe).
        let handler = AssertUnwindSafe(|| self.0.handle_request(http_request));

        let http_response = std::panic::catch_unwind(handler);

        match http_response {
            Ok(Ok(http_response)) => {
                // TODO: Write headers without allocating
                let headers: Vec<_> = http_response.headers().iter().collect();

                req.create_response(&headers, http_response.body())?;
            }
            Ok(Err(_err)) => {
                // FIXME: Proper error
                req.create_response(&[("Content-Type", "text/plain")], "Errored.\n")?;
            }
            Err(panic_payload) => {
                // FIXME: Log error

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
    F: Fn(Request<BodyReader>) -> UnitResult<Response<Vec<u8>>>,
    F: UnwindSafe + 'static,
{
    fn handle_request(&mut self, req: Request<BodyReader>) -> UnitResult<Response<Vec<u8>>> {
        self(req)
    }
}
