use core::ffi::c_int;

use magenboy_common::audio::AudioResampler;
use magenboy_common::audio::ManualAudioResampler;
use magenboy_core::AudioDevice;

use magenboy_core::GfxDevice;

use magenboy_core;

use magenboy_core::JoypadProvider;
use magenboy_core::keypad::button::Button;

pub type JoypadProviderCallback = unsafe extern "C" fn() -> u64;

// Copied from libnx definitions
const fn bit(index: u64) -> u64 { 1 << index }
const JOYCON_A: u64     = bit(0);
const JOYCON_B: u64     = bit(1);
const JOYCON_PLUS: u64  = bit(10);
const JOYCON_MINUS: u64 = bit(11);
const JOYCON_LEFT: u64  = bit(12);
const JOYCON_UP: u64    = bit(13);
const JOYCON_RIGHT: u64 = bit(14);
const JOYCON_DOWN: u64  = bit(15);

pub(crate) struct NxJoypadProvider{
    pub cb: JoypadProviderCallback
}

impl JoypadProvider for NxJoypadProvider {
    fn provide(&mut self, joypad: &mut magenboy_core::keypad::joypad::Joypad) {
        let joycons_state = unsafe{(self.cb)()};

        joypad.buttons[Button::A as usize]      = (joycons_state & JOYCON_A) != 0;
        joypad.buttons[Button::B as usize]      = (joycons_state & JOYCON_B) != 0;
        joypad.buttons[Button::Start as usize]  = (joycons_state & JOYCON_PLUS) != 0;
        joypad.buttons[Button::Select as usize] = (joycons_state & JOYCON_MINUS) != 0;
        joypad.buttons[Button::Up as usize]     = (joycons_state & JOYCON_UP) != 0;
        joypad.buttons[Button::Down as usize]   = (joycons_state & JOYCON_DOWN) != 0;
        joypad.buttons[Button::Right as usize]  = (joycons_state & JOYCON_RIGHT) != 0;
        joypad.buttons[Button::Left as usize]   = (joycons_state & JOYCON_LEFT) != 0;
    }
}

pub type GfxDeviceCallback = unsafe extern "C" fn(buffer:*const u16, width:c_int, height: c_int) -> ();

pub(crate) struct NxGfxDevice{
    pub cb: GfxDeviceCallback
}

impl GfxDevice for NxGfxDevice{
    fn swap_buffer(&mut self, buffer:&[magenboy_core::Pixel; magenboy_core::ppu::gb_ppu::SCREEN_HEIGHT * magenboy_core::ppu::gb_ppu::SCREEN_WIDTH]) {
        unsafe{(self.cb)(buffer.as_ptr(), magenboy_core::ppu::gb_ppu::SCREEN_WIDTH as c_int, magenboy_core::ppu::gb_ppu::SCREEN_HEIGHT as c_int)};
    }
}

pub type AudioDeviceCallback = unsafe extern "C" fn(buffer:*const magenboy_core::apu::audio_device::StereoSample, size:c_int) -> ();

pub(crate) struct NxAudioDevice{
    pub cb: AudioDeviceCallback,
    pub resampler: ManualAudioResampler,
}

impl AudioDevice for NxAudioDevice{
    fn push_buffer(&mut self, buffer:&[magenboy_core::apu::audio_device::StereoSample; magenboy_core::apu::audio_device::BUFFER_SIZE]) {
        let resampled = self.resampler.resample(buffer);
        unsafe{(self.cb)(resampled.as_ptr(), (resampled.len() * 2) as c_int)};
    }
}
