use std::env;
use std::path::PathBuf;

fn main() {
    let profile = std::env::var("PROFILE").unwrap();

    // Tell cargo to tell rustc to link the system Nginx unit
    // shared library.
    if profile == "debug" {
        println!("cargo:rustc-link-lib=unit-debug");
    } else {
        println!("cargo:rustc-link-lib=unit");
    }

    // Use vendored headers for libunit for docs.rs's builder, in order to
    // bypass the unit-dev dependency. This is only good enough for `cargo doc`
    // and cannot support a full build.
    let clang_args = if std::env::var("DOCS_RS").is_ok() {
        &["-I./.docs.rs/libunit-1.27.0"][..]
    } else {
        &[]
    };

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Use the vendored headers for docs.rs builds
        .clang_args(clang_args)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
