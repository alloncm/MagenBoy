use std::process::Command;

fn main() {
    let current_branch = std::fs::read_to_string("../.git/HEAD").unwrap();
    let current_commit_file = std::path::Path::new("../.git").join(&current_branch[5..]);
    let current_commit_file = current_commit_file.to_str().unwrap();
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed={}", current_commit_file);

    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    let version = env!("CARGO_PKG_VERSION");
    println!("cargo:rustc-env=MAGENBOY_VERSION={}", std::format!("{}-{}", version, git_hash));
}