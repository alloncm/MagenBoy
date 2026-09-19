use sdl2::sys::*;

use magenboy_core::keypad::joypad::{Joypad, NUM_OF_KEYS};

use crate::utils::get_sdl_error_message;

pub struct SdlJoypadProvider{
    mapping: [SDL_Scancode; NUM_OF_KEYS]
}

impl SdlJoypadProvider{
    pub fn new(mapping: [SDL_Scancode; NUM_OF_KEYS])->Self{
        Self{mapping}
    }

    pub fn provide(&self) -> Joypad {
        let mut joypad = Joypad::default();
        unsafe{
            let state = SDL_GetKeyboardState(std::ptr::null_mut());
            for i in 0..NUM_OF_KEYS{
                joypad.buttons[i] = *state.add(self.mapping[i] as usize) != 0;
            }
        }

        return joypad;
    }

    pub fn poll(&mut self) -> Joypad {
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
        return self.provide();
    }
}