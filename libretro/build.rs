use std::{env::var, path::Path};

// Writing a compatible .info file for retroarch.
// This file gives retroarch info about the core including which files it can to load.
// Without it retroarch wont allow the core to load any rom type.
fn main() {
    let mut executable_filename = var("CARGO_PKG_NAME").unwrap();
    // On unix .so files have lib prefix
    if var("CARGO_CFG_UNIX").is_ok(){
        executable_filename = "lib".to_owned() + &executable_filename;
    }
    let out_dir = var("OUT_DIR").unwrap();
    // Turns the out dir to the artifacts dir
    let mut info_filename = Path::new(&out_dir).to_path_buf();
    info_filename.pop();
    info_filename.pop();
    info_filename.pop();
    info_filename = info_filename.join(executable_filename);
    info_filename.set_extension("info");
    let version = magenboy_common::VERSION;
    let authors = var("CARGO_PKG_AUTHORS").unwrap();
    let content = std::format!(
r##"display_name = "MagenBoy - GameBoy & GameBoy Color"
categories = "Emulator"
authors = "{authors}"
supported_extensions = "gb|gbc"
license = "GPLv3"
display_version = "v{version}"
manufacturer = "Nintendo"
systemname = "Game Boy / Color"
description = "Cross platform Game Boy and Game Boy Color emulator"##);

    std::fs::write(info_filename, content).unwrap();
}