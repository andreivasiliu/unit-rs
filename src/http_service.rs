use http::{Request, Response};

use crate::{unit::UnitService, UnitRequest, UnitResult};

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
pub trait HttpService {
    fn handle_request(&mut self, _req: Request<Vec<u8>>) -> UnitResult<Response<Vec<u8>>>;
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
        let mut http_request_builder = Request::builder();

        for (name, value) in req.fields() {
            http_request_builder = http_request_builder.header(name, value);
        }

        // TODO: Read body from request
        let http_request = http_request_builder.body(Vec::new()).unwrap();

        let http_response = self.0.handle_request(http_request)?;

        // TODO: Write headers without allocating
        let headers: Vec<_> = http_response.headers().iter().collect();

        req.create_response(&headers, http_response.body())?;

        Ok(())
    }
}

impl<F> HttpService for F
where
    F: FnMut(Request<Vec<u8>>) -> UnitResult<Response<Vec<u8>>> + 'static,
{
    fn handle_request(&mut self, req: Request<Vec<u8>>) -> UnitResult<Response<Vec<u8>>> {
        self(req)
    }
}
