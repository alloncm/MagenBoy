pub mod emulation_menu;
pub mod joypad_menu;
pub mod mbc_handler;
pub mod mpmc_gfx_device;

use std::path::PathBuf;

use joypad_menu::*;
use lib_gb::keypad::joypad_provider::JoypadProvider;

pub const VERSION:&str = env!("CARGO_PKG_VERSION");

pub fn get_rom_selection<MR:MenuRenderer<PathBuf, String>, JP:MenuJoypadProvider + JoypadProvider>(roms_path:&str, menu_renderer:MR, jp:&mut JP)->String{
    let mut menu_options = Vec::new();
    let dir_entries = std::fs::read_dir(roms_path).expect(std::format!("Error openning the roms directory: {}",roms_path).as_str());
    for entry in dir_entries{
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(extension) = path.as_path().extension().and_then(std::ffi::OsStr::to_str){
            match extension {
                "gb" | "gbc"=>{
                    let filename = String::from(path.file_name().expect("Error should be a file").to_str().unwrap());
                    let option = MenuOption{value: path, prompt: filename};
                    menu_options.push(option);
                },
                _=>{}
            }
        }
    }
    let mut menu = JoypadMenu::new(&menu_options, String::from("Choose ROM"), menu_renderer);
    let result = menu.get_menu_selection(jp);

    return String::from(result.to_str().unwrap());
}

pub fn init_fern_logger()->Result<(), fern::InitError>{
    let fern_logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S.%f]"),
                record.level(),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?);

    fern_logger.apply()?;

    Ok(())
}