use magenboy_core::keypad::joypad::Joypad;

use crate::joypad_menu::MenuResult;


#[derive(Default, Clone, Copy)]
pub struct MenuOption<T, S:AsRef<str>>{
    pub value:T,
    pub prompt:S
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum EmulatorMenuOption{
    Resume,
    Restart,
    Shutdown
}

pub const GAME_MENU_OPTIONS:[MenuOption<EmulatorMenuOption, &str>;3] = [
    MenuOption{prompt:"Resume", value:EmulatorMenuOption::Resume},
    MenuOption{prompt:"Restart", value:EmulatorMenuOption::Restart}, 
    MenuOption{prompt:"Shutdown", value:EmulatorMenuOption::Shutdown}
];

cfg_if::cfg_if!{ if #[cfg(feature = "std")]{
    use std::path::PathBuf;

    use super::joypad_menu::JoypadMenu;

    pub struct MagenBoyMenu<'a> {
        menu: JoypadMenu<'a, EmulatorMenuOption, &'a str>
    }

    impl<'a> MagenBoyMenu<'a> {
        pub fn new(header:&'a str)->Self{
            Self { menu: JoypadMenu::new(&GAME_MENU_OPTIONS, header) }
        }

        pub fn get_game_menu_selection(&mut self, joypad: Joypad) -> MenuResult<'a, EmulatorMenuOption>{
            return self.menu.try_get_menu_selection(joypad);
        }

        pub fn get_rom_selection(&self, roms_path:&str, joypad: Joypad)->String{
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
            let result = self.menu.try_get_menu_selection(joypad);

            return String::from(result.to_str().unwrap());
        }

    }
}}