use core::arch::{global_asm, asm};

#[no_mangle]
static PERIPHERALS_BASE_ADDRESS:u32 = crate::peripherals::PERIPHERALS_BASE_ADDRESS as u32;

global_asm!(include_str!("boot_armv7a.s"));

extern "C"{
    // declared at startup assembly file
    pub fn hang_led()->!;
}

pub fn get_cpu_execution_mode()->u32{
    let mut mode:u32;
    unsafe{asm!("mrs {r}, cpsr", r = out(reg) mode)};
    // only the first 5 bits are relevant for the mode
    return mode & 0b1_1111;
}