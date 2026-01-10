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
    use super::joypad_menu::{MenuJoypadProvider, menu_renderer, JoypadMenu, MenuRenderer};

    pub struct MagenBoyMenu {
        header:String,
    }

    impl MagenBoyMenu {
        pub fn new(header:String)->Self{
            Self { header }
        }

        pub fn pop_game_menu(&mut self){
            match self.get_game_menu_selection(){
                EmulatorMenuOption::Resume => {},
                EmulatorMenuOption::Restart => state.running.store(false, std::sync::atomic::Ordering::Relaxed),
                EmulatorMenuOption::Shutdown => {
                    state.running.store(false, std::sync::atomic::Ordering::Relaxed);
                    state.exit.store(true, std::sync::atomic::Ordering::Relaxed);
                },
            }
        }

        fn get_game_menu_selection(&mut self)->&EmulatorMenuOption{
            let menu_renderer = menu_renderer::MenuRenderer::new();
            let mut menu = JoypadMenu::new(&GAME_MENU_OPTIONS, &self.header, menu_renderer);  
            return menu.get_menu_selection(&mut self.provider);
        }

    }

    pub fn get_rom_selection(roms_path:&str)->String{
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
}}