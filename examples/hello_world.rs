use unit_rs::{Unit, UnitRequest, UnitResult};

fn main() {
    let mut unit = Unit::new();

    unit.set_request_handler(request_handler);

    unit.run();
}

fn request_handler(req: UnitRequest) -> UnitResult<()> {
    // Create and send a response.
    let headers = &[("Content-Type", "text/plain")];
    req.create_response(headers, "Hello world!\n")?;

    Ok(())
}
