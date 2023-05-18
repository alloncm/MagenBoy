const LD_SCRIPT_PATH:&str = "link.ld";

fn main(){
    println!("cargo:rerun-if-changed={}", LD_SCRIPT_PATH);
}