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

    use magenboy_core::keypad::joypad::Joypad;

    use super::joypad_menu::{MenuResult, JoypadMenu};

    pub struct MagenBoyMenu<'a> {
        game_menu: JoypadMenu<'a, EmulatorMenuOption, &'a str>,
        rom_menu: Option<JoypadMenu<'a, PathBuf, String>>
    }

    impl<'a> MagenBoyMenu<'a> {
        pub fn new(header:&'a str, menu_options: Option<&'a [MenuOption<PathBuf, String>]>)->Self{
            let rom_menu = menu_options.and_then(|options| Some(JoypadMenu::new(options, header.to_string())));

            Self { game_menu: JoypadMenu::new(&GAME_MENU_OPTIONS, header), rom_menu }
        }

        pub fn get_game_menu_selection(&mut self, joypad: Joypad) -> MenuResult<'a, EmulatorMenuOption>{
            return self.game_menu.try_get_menu_selection(joypad);
        }

        pub fn get_rom_selection(&mut self, joypad: Joypad)->MenuResult<'a, PathBuf>{
            return self.rom_menu.as_mut().unwrap().try_get_menu_selection(joypad);
        }
    }

    pub fn read_roms_menu_options(roms_path: &str) -> Vec<MenuOption<PathBuf, String>> {
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

        return menu_options;
    }
}}