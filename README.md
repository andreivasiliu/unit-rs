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
          "processes": 4,
      }
  }
}
```

See the [`examples/request_info.rs`](examples/request_info.rs) example for a
more in-depth example, and the `deploy.sh` script for an example `curl` command.


## Building

In order to build, the library requires `libclang` (needed by `bindgen`) and `unit-dev` (which provides `libunit.a` and its headers).

Most distributions will have a `libclang-dev` package (or similar), while
`unit-dev` must be installed from Unit's own repositories linked in their
[installation guide](http://unit.nginx.org/installation/).

Note that Nginx Unit requires the server and applicaton to have the same
version; an application compiled with a `libunit` from an older or newer version
of Nginx Unit will not work.


## Benchmarks

A test on a Ryzen 7 CPU with the [`wrk`] benchmarking tool shows Unit reaching
~250000 requests per second, mostly maxing out on the `Unit` server and the
`wrk` tool itself.

For comparison, a classic [Nginx] server serving static files reached ~200000 requests per second (although note that the classic Nginx has significantly more
features).

[`wrk`]: https://github.com/wg/wrk
[Nginx]: https://nginx.org/
