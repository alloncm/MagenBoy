use sdl2::sys::*;
use magenboy_core::keypad::{joypad::{Joypad, NUM_OF_KEYS}, joypad_provider::JoypadProvider};
use magenboy_common::joypad_menu::MenuJoypadProvider;
use super::utils::get_sdl_error_message;


pub struct SdlJoypadProvider{
    mapping: [SDL_Scancode; NUM_OF_KEYS],

    // According to the docs events should be pumped from the main thread (the thread that initializes SDL) and its unsound to pump them from other threads
    // Since this struct is used from various threads Im allowing it to use both 
    poll_events:bool
}

impl SdlJoypadProvider{
    pub fn new(mapping: [SDL_Scancode; NUM_OF_KEYS], poll_events: bool)->Self{
        Self{mapping, poll_events}
    }
}

impl JoypadProvider for SdlJoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad) {
        unsafe{
            if self.poll_events {
                SDL_PumpEvents();
            }
            let state = SDL_GetKeyboardState(std::ptr::null_mut());
            for i in 0..NUM_OF_KEYS{
                joypad.buttons[i] = *state.add(self.mapping[i] as usize) != 0;
            }
        }
    }
}

impl MenuJoypadProvider for SdlJoypadProvider{
    fn poll(&mut self, joypad:&mut Joypad) {
        joypad.buttons.fill(false);
        unsafe{
            loop{
                let mut event = std::mem::MaybeUninit::<SDL_Event>::uninit();
                if SDL_WaitEvent(event.as_mut_ptr()) == 0{
                    std::panic!("SDL_Error: {}", get_sdl_error_message());
                }
                let event = event.assume_init();
                if event.type_ == SDL_EventType::SDL_KEYDOWN as u32 || event.type_ == SDL_EventType::SDL_KEYUP as u32 {
                    break;
                }
            }
        }
        self.provide(joypad);
    }
}