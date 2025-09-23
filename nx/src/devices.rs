use core::ffi::c_int;

use magenboy_common::{audio::{ManualAudioResampler, AudioResampler}, joypad_menu::MenuJoypadProvider, menu::Turbo};
use magenboy_core::{AudioDevice, GfxDevice, self, JoypadProvider, keypad::button::Button, GB_FREQUENCY};

pub type JoypadProviderCallback = unsafe extern "C" fn() -> u64;
pub type PollJoypadProviderCallback = unsafe extern "C" fn() -> u64;

// Copied from libnx definitions
const fn bit(index: u64) -> u64 { 1 << index }
const JOYCON_A: u64     = bit(0);
const JOYCON_B: u64     = bit(1);
const JOYCON_X: u64     = bit(2);
const JOYCON_Y: u64     = bit(3);
const JOYCON_PLUS: u64  = bit(10);
const JOYCON_MINUS: u64 = bit(11);
const JOYCON_LEFT: u64  = bit(12);
const JOYCON_UP: u64    = bit(13);
const JOYCON_RIGHT: u64 = bit(14);
const JOYCON_DOWN: u64  = bit(15);

pub(crate) struct NxJoypadProvider{
    pub provider_cb: JoypadProviderCallback,
    pub poll_cb: PollJoypadProviderCallback
}

impl NxJoypadProvider{
    fn update_state(joypad: &mut magenboy_core::keypad::joypad::Joypad, joycons_state: u64) {
        joypad.buttons[Button::A as usize]      = (joycons_state & JOYCON_A) != 0 || (joycons_state & JOYCON_X) != 0;
        joypad.buttons[Button::B as usize]      = (joycons_state & JOYCON_B) != 0 || (joycons_state & JOYCON_Y) != 0;
        joypad.buttons[Button::Start as usize]  = (joycons_state & JOYCON_PLUS) != 0;
        joypad.buttons[Button::Select as usize] = (joycons_state & JOYCON_MINUS) != 0;
        joypad.buttons[Button::Up as usize]     = (joycons_state & JOYCON_UP) != 0;
        joypad.buttons[Button::Down as usize]   = (joycons_state & JOYCON_DOWN) != 0;
        joypad.buttons[Button::Right as usize]  = (joycons_state & JOYCON_RIGHT) != 0;
        joypad.buttons[Button::Left as usize]   = (joycons_state & JOYCON_LEFT) != 0;
    }
}

impl JoypadProvider for NxJoypadProvider {
    fn provide(&mut self, joypad: &mut magenboy_core::keypad::joypad::Joypad) {
        let joycons_state = unsafe{(self.provider_cb)()};
        Self::update_state(joypad, joycons_state);
    }
}

impl MenuJoypadProvider for NxJoypadProvider {
    fn poll(&mut self, joypad:&mut magenboy_core::keypad::joypad::Joypad) {
        let joycon_state = unsafe{(self.poll_cb)()};
        Self::update_state(joypad, joycon_state);
    }
}

pub type GfxDeviceCallback = unsafe extern "C" fn(buffer:*const u16) -> ();

pub(crate) struct NxGfxDevice<'a>{
    pub cb: GfxDeviceCallback,
    pub turbo: &'a Turbo
}

impl<'a> GfxDevice for NxGfxDevice<'a>{
    fn swap_buffer(&mut self, buffer:&[magenboy_core::Pixel; magenboy_core::ppu::gb_ppu::SCREEN_HEIGHT * magenboy_core::ppu::gb_ppu::SCREEN_WIDTH]) {
        if self.turbo.update_and_check() {
            unsafe{(self.cb)(buffer.as_ptr())}; 
        }
    }
}

// Since StereoSample is repr(C, packed), we can safely transmute it to a slice of i16
// SAFETY: StereoSample is repr(C, packed)
pub type AudioDeviceCallback = unsafe extern "C" fn(buffer:*const magenboy_core::apu::audio_device::Sample, size:c_int) -> ();

pub(crate) struct NxAudioDevice<'a>{
    pub cb: AudioDeviceCallback,
    pub resampler: ManualAudioResampler,
    pub turbo: &'a Turbo
}

impl<'a> AudioDevice for NxAudioDevice<'a>{
    fn push_buffer(&mut self, buffer:&[magenboy_core::apu::audio_device::StereoSample; magenboy_core::apu::audio_device::BUFFER_SIZE]) {
        self.resampler.set_original_frequency(self.turbo.get_factor() * GB_FREQUENCY);
        let resampled = self.resampler.resample(buffer);
        let resampled_ptr = resampled.as_ptr() as *const magenboy_core::apu::audio_device::Sample;
        unsafe{(self.cb)(resampled_ptr, (resampled.len() * 2) as c_int)};
    }
}
