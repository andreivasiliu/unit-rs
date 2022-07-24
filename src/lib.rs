//! # unit-rs
//!
//! `unit-rs` is a safe wrapper around the C `libunit` library from [NGINX Unit],
//! which allows creating Unit applications in Rust.
//!
//! [NGINX Unit]: https://unit.nginx.org/
//!
//! ## Example
//!
//! ```no_run
//! use unit_rs::{Unit, Request};
//!
//! fn main() {
//!     let mut unit = Unit::new().unwrap();
//!
//!     unit.set_request_handler(|req: Request<'_>| {
//!         let headers = &[("Content-Type", "text/plain")];
//!         let body = "Hello world!\n";
//!         req.send_response(200, headers, body)?;
//!     
//!         Ok(())
//!     });
//!
//!     unit.run();
//! }
//! ```
//!
//! ## Features
//!
//! Currently not all features are supported, but enough are available to
//! inspect all aspects of a request and create a response.
//!
//! This library is also capable of multi-threading by creating additional
//! instances of [`Unit`].
//!
//! When the `http` feature enabled, the [`http::HttpHandler`] adapter can be
//! used to write handlers using types from the [`http`](https://docs.rs/http)
//! crate.
//!
//! ## Missing features
//!
//! WebSockets support is not yet implemented.
//!
//! A callback for inspecting a request header (and potentially closing the
//! request) before Unit buffers the whole request body is not yet available.
//!
//! There is currently no way to perform asynchronous handling of requests.
//! Handlers with expensive computations or blocking IO will block the whole
//! thread context.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub mod http;
mod nxt_unit;
mod request;
mod response;
mod unit;

pub use error::{UnitError, UnitInitError, UnitResult};
pub use request::{BodyReader, Request};
pub use response::{BodyWriter, Response};
pub use unit::{Unit, UnitService};
