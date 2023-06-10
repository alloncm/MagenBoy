use std::sync::{atomic::AtomicBool, Mutex};
use lib_gb::{ppu::gfx_device::GfxDevice, keypad::joypad_provider::JoypadProvider};

use crate::joypad_menu::{MenuOption, MenuJoypadProvider, joypad_gfx_menu, JoypadMenu};

enum EmulatorMenuOption{
    Resume,
    Restart,
    Shutdown
}

const GAME_MENU_OPTIONS:[MenuOption<EmulatorMenuOption, &str>;3] = [
    MenuOption{prompt:"Resume", value:EmulatorMenuOption::Resume},
    MenuOption{prompt:"Restart", value:EmulatorMenuOption::Restart}, 
    MenuOption{prompt:"Shutdown", value:EmulatorMenuOption::Shutdown}
];

pub struct MagenBoyState{
    // Use atomic bool, normal bool doesnt works on arm (probably cause of the memory model)
    pub running:AtomicBool,
    pub pause:AtomicBool,
    pub exit:AtomicBool,
    pub state_mutex:Mutex<()>
}

impl MagenBoyState{
    pub const fn new() -> Self {
        Self { running: AtomicBool::new(true), pause: AtomicBool::new(false), exit: AtomicBool::new(false), state_mutex: Mutex::new(()) }
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

    pub fn pop_game_menu<GFX:GfxDevice>(&mut self, state:&MagenBoyState, gfx_device:&mut GFX, receiver:crossbeam_channel::Receiver<usize>){
        match self.get_game_menu_selection(state, gfx_device, receiver){
            EmulatorMenuOption::Resume => {},
            EmulatorMenuOption::Restart => state.running.store(false, std::sync::atomic::Ordering::Relaxed),
            EmulatorMenuOption::Shutdown => {
                state.running.store(false, std::sync::atomic::Ordering::Relaxed);
                state.exit.store(true, std::sync::atomic::Ordering::Relaxed);
            },
        }
    }

    fn get_game_menu_selection<GFX:GfxDevice>(&mut self, state:&MagenBoyState,gfx_device:&mut GFX, emulation_framebuffer_channel:crossbeam_channel::Receiver<usize>)->&EmulatorMenuOption{
        let menu_renderer = joypad_gfx_menu::GfxDeviceMenuRenderer::new(gfx_device);
    
        let mut menu = JoypadMenu::new(&GAME_MENU_OPTIONS, &self.header, menu_renderer);  
    
        // lock the mutex here to sync the 2 threads
        state.pause.store(true, std::sync::atomic::Ordering::SeqCst);
        loop{
            if let Ok(_lock) = state.state_mutex.try_lock(){
                let selection = menu.get_menu_selection(&mut self.provider);
                state.pause.store(false, std::sync::atomic::Ordering::SeqCst);
                return selection;
            }else{
                // try recv in order to clear frames from the channel 
                // in order to not block the emualtion thread and allow it to finish the frame
                let _ = emulation_framebuffer_channel.try_recv();
            }
        }
    }
    
}