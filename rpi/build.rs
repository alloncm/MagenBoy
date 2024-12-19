#[cfg(not(feature = "os"))]
mod config{
    pub const RPI_ENV_VAR_NAME:&'static str = "RPI";
    pub const LD_SCRIPT_PATH:&str = "src/bin/baremetal/link.ld";
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
        let crate_manifest_path = env!("CARGO_MANIFEST_DIR");
        let ld_script_path = std::path::Path::new(crate_manifest_path).join(config::LD_SCRIPT_PATH);
        let ld_script_path = ld_script_path.to_str().unwrap();

        println!("cargo:rerun-if-changed={}", ld_script_path);
        println!("cargo:rerun-if-env-changed={}", config::RPI_ENV_VAR_NAME);

        // Linker script
        println!("cargo:rustc-link-arg-bin=baremetal={}", ld_script_path);

        // Creates config.txt
        let out_dir = std::env::var("OUT_DIR").unwrap();
        // Turns the out dir to the artifacts dir
        let mut config_file_path = std::path::Path::new(&out_dir).to_path_buf();
        config_file_path.pop();
        config_file_path.pop();
        config_file_path.pop();
        config_file_path = config_file_path.join("config.txt");
        std::fs::write(config_file_path, config::CONFIG_TXT_CONTENT).unwrap();

        // Add the cfg option `rpi` with that value of the env var `RPI`
        let rpi_version = std::env::var(config::RPI_ENV_VAR_NAME)
            .expect(std::format!("{} env must be set", config::RPI_ENV_VAR_NAME).as_str());
        println!("cargo:rustc-cfg=rpi=\"{}\"", rpi_version);

        // Silent warnings for this cfg 
        println!("cargo::rustc-check-cfg=cfg(rpi, values(\"4\", \"2\"))");
        println!("cargo::rustc-check-cfg=cfg(rpi)");
    }
}