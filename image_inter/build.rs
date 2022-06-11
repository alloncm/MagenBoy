fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/scale.c");
    // Use the `cc` crate to build a C file and statically link it.

    cc::Build::new()
        .warnings(true)
        .extra_warnings(true)
        .warnings_into_errors(true)
        .file("src/scale.c")
        .compile("scale");
}
