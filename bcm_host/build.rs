fn main() {
    // Checking for rpi
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    println!("cargo:rustc-link-lib=bcm_host");
}