#[cfg(not(feature = "os"))]
mod config{
    pub const RPI_ENV_VAR_NAME:&'static str = "RPI";
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
    #[cfg(feature = "os")]
    {
        println!("cargo:rustc-link-lib=bcm_host");
        println!("cargo:rustc-link-search=/opt/vc/lib");
    }
    #[cfg(not(feature = "os"))]
    {
        println!("cargo:rerun-if-changed={}", config::LD_SCRIPT_PATH);
        println!("cargo:rerun-if-env-changed={}", config::RPI_ENV_VAR_NAME);

        // Creates config.txt
        std::fs::write(config::CONFIG_FILE_PATH, config::CONFIG_TXT_CONTENT).unwrap();

        // Add the cfg option `rpi` with that value of the env var `RPI`
        let rpi_revision = std::env::var(config::RPI_ENV_VAR_NAME).expect("RPI env must be set");
        println!("cargo:rustc-cfg=rpi=\"{}\"", rpi_revision);
    }
}