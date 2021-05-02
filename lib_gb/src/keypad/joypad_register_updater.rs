use super::{button::Button, joypad::Joypad};
use crate::{
    mmu::memory::UnprotectedMemory,
    utils::{
        bit_masks::{set_bit_u8, BIT_4_MASK, BIT_5_MASK},
        memory_registers::JOYP_REGISTER_ADDRESS,
    },
};

pub fn update_joypad_registers(joypad: &Joypad, memory: &mut impl UnprotectedMemory) {
    let mut state = memory.read_unprotected(JOYP_REGISTER_ADDRESS);

    let buttons = (state & BIT_5_MASK) == 0;
    let directions = (state & BIT_4_MASK) == 0;

    if buttons {
        set_bit_u8(&mut state, 0, !joypad.buttons[Button::A as usize]);
        set_bit_u8(&mut state, 1, !joypad.buttons[Button::B as usize]);
        set_bit_u8(&mut state, 2, !joypad.buttons[Button::Select as usize]);
        set_bit_u8(&mut state, 3, !joypad.buttons[Button::Start as usize]);
    }
    if directions {
        set_bit_u8(&mut state, 0, !joypad.buttons[Button::Right as usize]);
        set_bit_u8(&mut state, 1, !joypad.buttons[Button::Left as usize]);
        set_bit_u8(&mut state, 2, !joypad.buttons[Button::Up as usize]);
        set_bit_u8(&mut state, 3, !joypad.buttons[Button::Down as usize]);
    }

    memory.write_unprotected(JOYP_REGISTER_ADDRESS, state);
}
