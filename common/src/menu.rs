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
    use std::{sync::atomic::AtomicBool, path::PathBuf};
    use super::joypad_menu::{MenuJoypadProvider, joypad_gfx_menu, JoypadMenu, MenuRenderer};

    pub struct MagenBoyState{
        // Use atomic bool, normal bool doesnt works on arm (probably cause of the memory model)
        pub running:AtomicBool,
        pub exit:AtomicBool,
    }

    impl MagenBoyState{
        pub const fn new() -> Self {
            Self { running: AtomicBool::new(true), exit: AtomicBool::new(false) }
        }
    }

    pub struct MagenBoyMenu<JP:JoypadProvider + MenuJoypadProvider>{
        header:String,
        provider:JP,
    }

    impl<JP:JoypadProvider + MenuJoypadProvider> MagenBoyMenu<JP> {
        pub fn new(provider:JP, header:String)->Self{
            Self { provider, header }
        }

        pub fn pop_game_menu<GFX:GfxDevice>(&mut self, state:&MagenBoyState, gfx_device:&mut GFX){
            match self.get_game_menu_selection(gfx_device){
                EmulatorMenuOption::Resume => {},
                EmulatorMenuOption::Restart => state.running.store(false, std::sync::atomic::Ordering::Relaxed),
                EmulatorMenuOption::Shutdown => {
                    state.running.store(false, std::sync::atomic::Ordering::Relaxed);
                    state.exit.store(true, std::sync::atomic::Ordering::Relaxed);
                },
            }
        }

        fn get_game_menu_selection<GFX:GfxDevice>(&mut self, gfx_device:&mut GFX)->&EmulatorMenuOption{
            let menu_renderer = joypad_gfx_menu::GfxDeviceMenuRenderer::new(gfx_device);
            let mut menu = JoypadMenu::new(&GAME_MENU_OPTIONS, &self.header, menu_renderer);  
            return menu.get_menu_selection(&mut self.provider);
        }

    }

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
}}