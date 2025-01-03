#![cfg_attr(not(feature = "dbg"), no_std)]

pub mod cpu;
pub mod machine;
pub mod ppu;
pub mod mmu;
pub mod keypad;
pub mod apu;
pub mod timer;
pub mod utils;
#[cfg(feature = "dbg")]
pub mod debugger;

pub use {
    machine::{gameboy::GameBoy, Mode},
    ppu::gfx_device::*,
    apu::audio_device::AudioDevice,
    keypad::joypad_provider::JoypadProvider,
    utils::GB_FREQUENCY, 
    mmu::external_memory_bus::{Bootrom, GB_BOOT_ROM_SIZE, GBC_BOOT_ROM_SIZE}
};
