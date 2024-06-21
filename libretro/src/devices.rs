use std::{ffi::c_void, mem::size_of};

use libretro_sys::*;

use magenboy_common::audio::*;
use magenboy_core::{apu::audio_device::*, keypad::{button::*, joypad::*, joypad_provider::*}, ppu::{gb_ppu::*, gfx_device::*}, GB_FREQUENCY};

use super::RETRO_CORE_CTX;

pub struct RetroGfxDevice;
impl GfxDevice for RetroGfxDevice{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        unsafe{(RETRO_CORE_CTX.video_cb.unwrap())(buffer.as_ptr() as *const c_void, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, SCREEN_WIDTH * size_of::<Pixel>())};
    }
}

pub struct RetroJoypadProvider;
impl JoypadProvider for RetroJoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad) {
        unsafe{
            (RETRO_CORE_CTX.input_poll_cb.unwrap())();

            let input_cb: unsafe extern fn(port:u32, device:u32, index:u32, id:u32) -> i16 = RETRO_CORE_CTX.input_cb.unwrap();
            
            joypad.buttons[Button::A as usize]      = input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_A) != 0 || input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_X) != 0;
            joypad.buttons[Button::B as usize]      = input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_B) != 0 || input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_Y) != 0;
            joypad.buttons[Button::Start as usize]  = input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_START) != 0;
            joypad.buttons[Button::Select as usize] = input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_SELECT) != 0;
            joypad.buttons[Button::Up as usize]     = input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_UP) != 0;
            joypad.buttons[Button::Down as usize]   = input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_DOWN) != 0;
            joypad.buttons[Button::Right as usize]  = input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_RIGHT) != 0;
            joypad.buttons[Button::Left as usize]   = input_cb(0, DEVICE_JOYPAD, 0, DEVICE_ID_JOYPAD_LEFT) != 0;
        }
    }
}

pub struct RetroAudioDevice{
    resampler: ManualAudioResampler
}

static mut DYNAMIC_AUDIO_BUFFER: Vec<StereoSample> = Vec::new();
impl RetroAudioDevice{
    pub const OUTPUT_FREQUENCY:u32 = 48000;
}

impl Default for RetroAudioDevice{
    fn default()->Self{
        Self { resampler: ManualAudioResampler::new(GB_FREQUENCY, Self::OUTPUT_FREQUENCY) }
    }
}

impl AudioDevice for RetroAudioDevice{
    fn push_buffer(&mut self, buffer:&[StereoSample; BUFFER_SIZE]) {
        let mut resampled = self.resampler.resample(buffer);
        unsafe{DYNAMIC_AUDIO_BUFFER.append(&mut resampled)};
    }
}

impl RetroAudioDevice{
    pub unsafe fn push_audio_buffer_to_libretro(){
        let mut remaining_frames = DYNAMIC_AUDIO_BUFFER.len();
        let mut buffer_pos_ptr = DYNAMIC_AUDIO_BUFFER.as_ptr() as *const Sample;
        while remaining_frames > 0 {
            let uploaded_frames = (RETRO_CORE_CTX.audio_cb.unwrap())(buffer_pos_ptr, remaining_frames);
            remaining_frames -= uploaded_frames;
            buffer_pos_ptr = buffer_pos_ptr.add(uploaded_frames);
        }
    
        DYNAMIC_AUDIO_BUFFER.clear();
    }
}