//! # unit-rs
//! 
//! `unit-rs` is a safe wrapper around the C `libunit` library from [Nginx Unit],
//! which allows creating Unit applications in Rust.
//! 
//! [Nginx Unit]: https://unit.nginx.org/
//! 
//! Currently very few features are supported, but enough are available to
//! inspect all aspects of a request and create a response.
//! 
//! ## Example
//! 
//! ```no_run
//! use unit_rs::Unit;
//! 
//! fn main() {
//!     let mut unit = Unit::new();
//! 
//!     unit.set_request_handler(|req| {
//!         let headers = &[("Content-Type", "text/plain")];
//!         let body = "Hello world!\n";
//!         req.create_response(headers, body)?;
//!     
//!         Ok(())
//!     });
//! 
//!     unit.run();
//! }
//! ```

mod nxt_unit;
mod request;
mod response;
mod unit;

pub use request::UnitRequest;
pub use response::UnitResponse;
pub use unit::{Unit, UnitResult, UnitError};
