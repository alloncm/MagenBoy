#![cfg_attr(not(feature = "os"), no_std)]

#[cfg(all(feature = "os", any(feature = "rpi4",feature = "rpi2")))]
core::compile_error!("rpiX features cant be combined with the os feature");

pub mod configuration;
pub mod peripherals;
pub mod drivers;
cfg_if::cfg_if!{ if #[cfg(not(feature = "os"))]{
    pub mod syncronization;
    mod delay;
}}

use magenboy_core::apu::audio_device::*;

pub struct BlankAudioDevice;
impl AudioDevice for BlankAudioDevice{
    fn push_buffer(&mut self, _buffer:&[StereoSample; BUFFER_SIZE]) {}
}