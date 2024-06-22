use sdl2::sys::*;
use magenboy_core::{keypad::{joypad::{Joypad, NUM_OF_KEYS}, joypad_provider::JoypadProvider, button::Button}, utils::create_array};
use magenboy_common::joypad_menu::MenuJoypadProvider;
use super::utils::get_sdl_error_message;


pub struct SdlJoypadProvider{
    keyborad_state: [*const u8;NUM_OF_KEYS],
}

impl SdlJoypadProvider{
    pub fn new<F:Fn(&Button)->SDL_Scancode>(mapper:F)->Self{
        let keyboard_ptr = unsafe{SDL_GetKeyboardState(std::ptr::null_mut())};
        let mut counter:u8 = 0;
        let init_lambda = ||{
            let button:Button = unsafe{std::mem::transmute(counter)};
            let result = unsafe{keyboard_ptr.offset(mapper(&button) as isize)};
            counter += 1;
            return result;
        };
        let state:[*const u8; NUM_OF_KEYS] = create_array(init_lambda);
        
        return Self{keyborad_state: state}
    }
}

impl JoypadProvider for SdlJoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad) {
        unsafe{
            SDL_PumpEvents();

            for i in 0..NUM_OF_KEYS{
                joypad.buttons[i] = *self.keyborad_state[i] != 0;
            }
        }
    }
}

impl MenuJoypadProvider for SdlJoypadProvider{
    fn poll(&mut self, mut joypad:&mut Joypad) {
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
        self.provide(&mut joypad);
    }
}