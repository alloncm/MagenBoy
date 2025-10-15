use core::sync::atomic::{AtomicBool, Ordering, AtomicU32};

#[derive(Default, Clone, Copy)]
pub struct MenuOption<T, S:AsRef<str>>{
    pub value:T,
    pub prompt:S
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum EmulatorMenuOption{
    Resume = 0,
    Restart = 1,
    Shutdown = 2,
    Turbo = 3,
}

pub const GAME_MENU_OPTIONS:[MenuOption<EmulatorMenuOption, &str>; 4] = [
    MenuOption{prompt:"Resume", value:EmulatorMenuOption::Resume},
    MenuOption{prompt:"Turbo", value:EmulatorMenuOption::Turbo},
    MenuOption{prompt:"Restart", value:EmulatorMenuOption::Restart}, 
    MenuOption{prompt:"Shutdown", value:EmulatorMenuOption::Shutdown}
];

pub struct Turbo {
    pub enabled: AtomicBool,
    factor: u32,
    counter: AtomicU32,
}

impl Turbo {
    pub const fn new(factor: u32) -> Self {
        Self { enabled: AtomicBool::new(false), factor, counter: AtomicU32::new(0) }
    }

    pub fn toggle(&self) {
        let new_turbo_state = !self.enabled.load(Ordering::Relaxed);
        self.enabled.store(new_turbo_state, Ordering::Relaxed);
        log::info!("Turbo mode is: {new_turbo_state}");
    }

    /// Updates the internal counter and returns true if a frame should be rendered
    /// Returns true if turbo is disabled
    pub fn update_and_check(&self) -> bool {
        if self.enabled.load(Ordering::Relaxed) {
            let value = self.counter.load(Ordering::SeqCst);
            let updated_value = (value + 1) % self.factor;
            self.counter.store(updated_value, Ordering::SeqCst);
            return updated_value == 0;
        } 
        return true;
    }

    pub fn get_factor(&self) -> u32 {
        if self.enabled.load(Ordering::Relaxed) {
            self.factor
        } else {
            1
        }
    }
}


cfg_if::cfg_if!{ if #[cfg(feature = "std")]{
    use std::{sync::Mutex, path::PathBuf};
    
    use super::joypad_menu::{MenuJoypadProvider, joypad_gfx_menu, JoypadMenu, MenuRenderer};

    use crate::{GfxDevice, JoypadProvider};

    pub struct MagenBoyState{
        // Use atomic bool, normal bool doesnt works on arm (probably cause of the memory model)
        pub running: AtomicBool,
        pub turbo: Turbo,
        pub pause: AtomicBool,
        pub exit: AtomicBool,
        pub state_mutex: Mutex<()>
    }

    impl MagenBoyState{
        pub const fn new(turbo_factor: u32) -> Self {
            Self { 
                running: AtomicBool::new(true), 
                turbo: Turbo::new(turbo_factor),
                pause: AtomicBool::new(false), 
                exit: AtomicBool::new(false), 
                state_mutex: Mutex::new(()) 
            }
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

        pub fn pop_game_menu<GFX: GfxDevice>(
            &mut self,
            state: &MagenBoyState,
            gfx_device: &mut GFX, 
        ) {
            match self.get_game_menu_selection(state, gfx_device){
                EmulatorMenuOption::Resume => {},
                EmulatorMenuOption::Turbo => state.turbo.toggle(),
                EmulatorMenuOption::Restart => state.running.store(false, std::sync::atomic::Ordering::Relaxed),
                EmulatorMenuOption::Shutdown => {
                    state.running.store(false, std::sync::atomic::Ordering::Relaxed);
                    state.exit.store(true, std::sync::atomic::Ordering::Relaxed);
                },
            }
        }

        fn get_game_menu_selection<GFX: GfxDevice>(
            &mut self,
            state: &MagenBoyState,
            gfx_device: &mut GFX,
        ) -> &EmulatorMenuOption {
            let menu_renderer = joypad_gfx_menu::GfxDeviceMenuRenderer::new(gfx_device);
        
            let mut menu = JoypadMenu::new(&GAME_MENU_OPTIONS, &self.header, menu_renderer);  
        
            // lock the mutex here to sync the 2 threads
            state.pause.store(true, std::sync::atomic::Ordering::SeqCst);
            loop{
                if let Ok(_lock) = state.state_mutex.try_lock(){
                    // Turn off turbo to have a regular speed menu
                    let last_turbo = state.turbo.enabled.swap(false, core::sync::atomic::Ordering::Relaxed);
                    let selection = menu.get_menu_selection(&mut self.provider);
                    // restore turbo state
                    state.turbo.enabled.store(last_turbo, core::sync::atomic::Ordering::Relaxed);
                    state.pause.store(false, std::sync::atomic::Ordering::SeqCst);
                    return selection;
                }
            }
        }

    }

    pub fn get_rom_selection<MR:MenuRenderer<PathBuf, String>, JP:MenuJoypadProvider + JoypadProvider>(
        roms_path: &str,
        menu_renderer: MR,
        jp:&mut JP
    ) -> String {
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