use crate::utils::bit_masks::*;
use super::{joypad_provider::JoypadProvider, joypad::Joypad, button::Button};


pub struct JoypadHandler<JP:JoypadProvider>{
    pub register:u8,

    joypad:Joypad,
    joypad_provider:JP,
}

impl<JP:JoypadProvider> JoypadHandler<JP>{
    pub fn new(provider: JP)->Self{
        Self{
            joypad_provider:provider,
            register:0xFF,
            joypad: Joypad::default()
        }
    }

    pub fn poll_joypad_state(&mut self){
        self.joypad_provider.provide(&mut self.joypad);

        let buttons = (self.register & BIT_5_MASK) == 0;
        let directions = (self.register & BIT_4_MASK) == 0;

        if buttons{
            flip_bit_u8(&mut self.register, 0, !self.joypad.buttons[Button::A as usize]);
            flip_bit_u8(&mut self.register, 1, !self.joypad.buttons[Button::B as usize]);
            flip_bit_u8(&mut self.register, 2, !self.joypad.buttons[Button::Select as usize]);
            flip_bit_u8(&mut self.register, 3, !self.joypad.buttons[Button::Start as usize]);
        }
        if directions{
            flip_bit_u8(&mut self.register, 0, !self.joypad.buttons[Button::Right as usize]);
            flip_bit_u8(&mut self.register, 1, !self.joypad.buttons[Button::Left as usize]);
            flip_bit_u8(&mut self.register, 2, !self.joypad.buttons[Button::Up as usize]);
            flip_bit_u8(&mut self.register, 3, !self.joypad.buttons[Button::Down as usize]);
        }
    }

    pub fn set_register(&mut self, value:u8){
        self.register &= 0b1100_1111;   // Reset bit 4 & 5
        self.register |= value & 0b0011_0000;   // Seting the bits
    }
}