use sdl2::sys::*;
use lib_gb::keypad::{
    joypad::Joypad,
    joypad_provider::JoypadProvider,
    button::Button
};

const PUMP_THRESHOLD:u32 = 500;

pub struct SdlJoypadProvider<F:Fn(Button)->SDL_Scancode>{
    mapper: F,
    pump_counter:u32,
    keyborad_state: *const u8
}

impl<F:Fn(Button)->SDL_Scancode> SdlJoypadProvider<F>{
    pub fn new(mapper:F)->Self{
        SdlJoypadProvider{
            mapper,
            pump_counter:PUMP_THRESHOLD,
            keyborad_state: unsafe{SDL_GetKeyboardState(std::ptr::null_mut())}
        }
    }
}

impl<F:Fn(Button)->SDL_Scancode> JoypadProvider for SdlJoypadProvider<F>{
    fn provide(&mut self, joypad:&mut Joypad) {
        let mapper = &(self.mapper);
        unsafe{
            self.pump_counter = (self.pump_counter + 1) % PUMP_THRESHOLD;
            if self.pump_counter == 0{
                SDL_PumpEvents();
            }

            joypad.buttons[Button::A as usize]      = *self.keyborad_state.offset(mapper(Button::A) as isize) != 0;
            joypad.buttons[Button::B as usize]      = *self.keyborad_state.offset(mapper(Button::B) as isize) != 0;
            joypad.buttons[Button::Start as usize]  = *self.keyborad_state.offset(mapper(Button::Start) as isize) != 0;
            joypad.buttons[Button::Select as usize] = *self.keyborad_state.offset(mapper(Button::Select) as isize) != 0;
            joypad.buttons[Button::Up as usize]     = *self.keyborad_state.offset(mapper(Button::Up) as isize) != 0;
            joypad.buttons[Button::Down as usize]   = *self.keyborad_state.offset(mapper(Button::Down) as isize) != 0;
            joypad.buttons[Button::Right as usize]  = *self.keyborad_state.offset(mapper(Button::Right) as isize) != 0;
            joypad.buttons[Button::Left as usize]   = *self.keyborad_state.offset(mapper(Button::Left) as isize) != 0;
        }
    }
}