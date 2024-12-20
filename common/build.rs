use std::process::Command;

fn main() {
    let git_path = std::path::Path::new("../.git");
    let head_path = git_path.join("HEAD");
    let current_branch = std::fs::read_to_string(&head_path).unwrap().replace("refs: ", "");
    let current_commit_file = git_path.join(&current_branch);
    println!("cargo:rerun-if-changed={}", head_path.to_str().unwrap());
    println!("cargo:rerun-if-changed={}", current_commit_file.to_str().unwrap());

    // let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap();
    // let git_hash = String::from_utf8(output.stdout).unwrap();
    let git_hash = "0";
    let version = env!("CARGO_PKG_VERSION");
    println!("cargo:rustc-env=MAGENBOY_VERSION={}", std::format!("{}-{}", version, git_hash));
}