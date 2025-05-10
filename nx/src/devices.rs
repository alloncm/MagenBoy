use core::ffi::c_int;

use magenboy_core::AudioDevice;

use magenboy_core::GfxDevice;

use magenboy_core;

use magenboy_core::JoypadProvider;

pub(crate) struct NxJoypadProvider;

impl JoypadProvider for NxJoypadProvider {
    fn provide(&mut self, _joypad: &mut magenboy_core::keypad::joypad::Joypad) {
        // TODO: implement
    }
}

pub type GfxDeviceCallback = extern "C" fn(buffer:*const u16, width:c_int, height: c_int) -> ();

pub(crate) struct NxGfxDevice{
    pub cb: GfxDeviceCallback
}

impl GfxDevice for NxGfxDevice{
    fn swap_buffer(&mut self, buffer:&[magenboy_core::Pixel; magenboy_core::ppu::gb_ppu::SCREEN_HEIGHT * magenboy_core::ppu::gb_ppu::SCREEN_WIDTH]) {
        (self.cb)(buffer.as_ptr(), magenboy_core::ppu::gb_ppu::SCREEN_WIDTH as c_int, magenboy_core::ppu::gb_ppu::SCREEN_HEIGHT as c_int);
    }
}

pub(crate) struct NxAudioDevice;

impl AudioDevice for NxAudioDevice{
    fn push_buffer(&mut self, _buffer:&[magenboy_core::apu::audio_device::StereoSample; magenboy_core::apu::audio_device::BUFFER_SIZE]) {
        // TODO: implement
    }
}
