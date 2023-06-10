#[cfg(not(feature = "std"))]
mod config{
    pub const LD_SCRIPT_PATH:&str = "src/bin/baremetal/link.ld";
    pub const CONFIG_FILE_PATH: &str = "config.txt";
    pub const CONFIG_TXT_CONTENT:&str = 
    "# configuration for the RPI
    arm_64bit=0 # boot to 32 bit mode

    # fast boot
    boot_delay=0
    disable_poe_fan=1
    disable_splash=1";
}

fn main(){
    #[cfg(feature = "std")]
    {
        println!("cargo:rustc-link-lib=bcm_host");
        println!("cargo:rustc-link-search=/opt/vc/lib");
    }
    #[cfg(not(feature = "std"))]
    {
        println!("cargo:rerun-if-changed={}", config::LD_SCRIPT_PATH);
        std::fs::write(config::CONFIG_FILE_PATH, config::CONFIG_TXT_CONTENT).unwrap();
    }
}