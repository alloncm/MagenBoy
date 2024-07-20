use sdl2::sys::*;
use magenboy_core::keypad::{joypad::{Joypad, NUM_OF_KEYS}, joypad_provider::JoypadProvider};
use magenboy_common::joypad_menu::MenuJoypadProvider;
use super::utils::get_sdl_error_message;


pub struct SdlJoypadProvider{
    mapping: [SDL_Scancode; NUM_OF_KEYS]
}

impl SdlJoypadProvider{
    pub fn new(mapping: [SDL_Scancode; NUM_OF_KEYS])->Self{
        Self{mapping}
    }
}

impl JoypadProvider for SdlJoypadProvider{
    // Events are pumped from the main thread (the thread that initializes SDL)
    // Its unsound to pump them from other threads
    fn provide(&mut self, joypad:&mut Joypad) {
        unsafe{
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
            let mut event = std::mem::MaybeUninit::<SDL_Event>::uninit();
            loop{
                if SDL_WaitEvent(event.as_mut_ptr()) == 0{
                    std::panic!("SDL_Error: {}", get_sdl_error_message());
                }
                let event = event.assume_init();
                if event.type_ == SDL_EventType::SDL_KEYUP as u32{
                    if let Some(index) = self.mapping.iter().position(|key|*key == event.key.keysym.scancode){
                        joypad.buttons[index] = true;
                    }
                    break;
                }
            }
        }
    }
}