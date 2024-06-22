#![cfg_attr(not(feature = "os"), no_std)]

#[cfg(all(feature = "os", rpi))]
core::compile_error!("The os feature and the rpi cfg value cant be set at the same time");

pub mod configuration;
pub mod peripherals;
pub mod drivers;
cfg_if::cfg_if!{ if #[cfg(not(feature = "os"))]{
    pub mod syncronization;
    pub mod delay;
}}

use magenboy_core::apu::audio_device::*;

pub const MENU_PIN_BCM:u8 = 3; // This pin is the turn on pin on thr RPI

pub struct BlankAudioDevice;
impl AudioDevice for BlankAudioDevice{
    fn push_buffer(&mut self, _buffer:&[StereoSample; BUFFER_SIZE]) {}
}