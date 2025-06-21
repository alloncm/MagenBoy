// Export values for the C application to use
fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut out_vars_path = std::path::Path::new(&out_dir).to_path_buf();
    out_vars_path.pop();
    out_vars_path.pop();
    out_vars_path.pop();
    std::fs::write(out_vars_path.join("version.txt"), version).expect("Unable to write version file");
    std::fs::write(out_vars_path.join("authors.txt"), authors).expect("Unable to write authors file");
}