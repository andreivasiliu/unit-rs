[![Crates.io](https://img.shields.io/crates/v/unit-rs)](https://crates.io/crates/unit-rs)
[![docs.rs](https://img.shields.io/docsrs/unit-rs)](https://docs.rs/unit-rs)
[![Crates.io](https://img.shields.io/crates/l/unit-rs)](./LICENSE)


# unit-rs

`unit-rs` is a safe wrapper around the `libunit` C library from [Nginx Unit]
which allows creating Unit applications in Rust.

[Nginx Unit]: https://unit.nginx.org/

Currently very few features are supported, but enough are available to inspect
all aspects of a request and create a response.


## Example

Add `unit-rs` as a dependency:

```toml
[dependencies]
unit-rs = "0.1"
```

And add the following to `src/main.rs`:

```rust
use unit_rs::Unit;

fn main() {
    let mut unit = Unit::new();

    unit.set_request_handler(|req| {
        let headers = &[("Content-Type", "text/plain")];
        let body = "Hello world!\n";
        req.create_response(headers, body)?;
    
        Ok(())
    });

    unit.run();
}
```

Once compiled, a running Nginx Unit server can be configured to use it with an
`external` application pointing to the binary's path:

```json
{
  "listeners": {
      "*:8080": {
          "pass": "applications/rustapp"
      }
  },
  "applications": {
      "rustapp": {
          "type": "external",
          "working_directory": "/path/to/package",
          "executable": "/path/to/package/hello_world",
          "processes": 4
      }
  }
}
```

See the [`examples/request_info.rs`](examples/request_info.rs) example for a
more in-depth example, and the `deploy.sh` script for an example `curl` command.


## Building

In order to build, the library requires `libclang` (needed by `bindgen`) and
`unit-dev` (which provides `libunit.a` and its headers).

Most distributions will have a `libclang-dev` package (or similar), while
`unit-dev` must be installed from Unit's own repositories linked in their
[installation guide](http://unit.nginx.org/installation/).

Note that Nginx Unit requires the server and applicaton to have the same
version; an application compiled with a `libunit` from an older or newer version
of Nginx Unit will not work.


## Safety

`unit-rs` attempts to be a safe wrapper by making all invalid uses of the 
`libunit` APIs impossible, and preventing all undefined behavior.

Some of this is achieved at the expense of some runtime performance, using
Rust's mandatory bounds-checking for all arrays.

For example, applications using `unit-rs` will experience runtime panics when:

* Reading more bytes from a request than available
* Writing more bytes to a response than allocated

However, Rust's type system also allows for many compile-time guarantees. When
using `unit-rs`, the following become either compile-time errors or impossible
to express:

* Moving Unit contexts or sharing request data between threads
* Accessing more fields/headers than available
* Storing pointers to request data in variables that outlive the request handler
  function
* Storing pointers to or into a shared memory buffer in variables that outlive
  the buffer
* Creating two responses to the same request
* Forgetting to finalize/close the request
* Creating/sending a shared memory buffer before previous buffers were sent or
  dealloated
* Sending a shared memory buffer that was already sent
* Sending uninitialized shared memory regions
* Forgetting to drop a shared memory buffer

Through the use of a global mutex, `unit-rs` will also ensure that any
additional multi-threaded Unit contexts will be spawned from a primary context,
and that the primary context outlives all secondary contexts.


## Benchmarks

A test on a Ryzen 7 CPU with the [`wrk`] benchmarking tool shows Unit reaching
~250000 requests per second, mostly maxing out on the `Unit` server and the
`wrk` tool itself.

For comparison, a classic [Nginx] server serving static files reached ~200000
requests per second (although note that the classic Nginx has significantly more
features).

[`wrk`]: https://github.com/wg/wrk
[Nginx]: https://nginx.org/
