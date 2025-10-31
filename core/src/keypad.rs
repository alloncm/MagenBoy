use crate::utils::bit_masks::{flip_bit_u8, BIT_4_MASK, BIT_5_MASK};

#[repr(u8)]
pub enum Button{
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Right,
    Left
}

pub const NUM_OF_KEYS: usize = 8;

#[derive(Default, Clone, Copy)]
pub struct Joypad {
    pub buttons: [bool; NUM_OF_KEYS]
}

pub struct JoypadHandler{
    register:u8,
    joypad:Joypad,
}

impl JoypadHandler{
    pub fn new()->Self{
        Self{
            register:0xFF,
            joypad: Joypad::default()
        }
    }

    pub fn update_joypad(&mut self, joypad: Joypad) {
        self.joypad = joypad;
    }

    pub fn get_register(&mut self) -> u8{
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

        return self.register;
    }
    
    pub fn set_register(&mut self, value:u8){
        self.register &= 0b1100_1111;   // Reset bit 4 & 5
        self.register |= value & 0b0011_0000;   // Seting the bits
    }
}