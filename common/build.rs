use std::process::Command;

fn main() {
    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    let version = env!("CARGO_PKG_VERSION");
    println!("cargo:rustc-env=MAGENBOY_VERSION={}", std::format!("{}-{}", version, git_hash));
}