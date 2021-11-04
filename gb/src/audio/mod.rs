pub mod multi_device_audio;
pub mod wav_file_audio_device;


cfg_if::cfg_if!{
    if #[cfg(feature = "push-audio")]{
        pub mod sdl_push_audio_device;
        pub type ChosenAudioDevice<AR> = sdl_push_audio_device::SdlPushAudioDevice<AR>;
    }
    else{
        pub mod sdl_pull_audio_device;
        pub type ChosenAudioDevice<AR> = sdl_pull_audio_device::SdlPullAudioDevice<AR>;
    }
}
cfg_if::cfg_if!{
    if #[cfg(feature = "sdl-resample")]{
        pub mod sdl_audio_resampler;
        pub type ChosenResampler = sdl_audio_resampler::SdlAudioResampler;
    }
    else{
        pub mod magen_audio_resampler;
        pub type ChosenResampler = magen_audio_resampler::MagenAudioResampler;
    }
}

use std::ffi::CStr;
use lib_gb::apu::audio_device::{AudioDevice, BUFFER_SIZE, Sample, StereoSample};
use sdl2::{libc::c_char, sys::SDL_GetError};

fn get_sdl_error_message()->&'static str{
    unsafe{
        let error_message:*const c_char = SDL_GetError();
        
        return CStr::from_ptr(error_message).to_str().unwrap();
    }
}

pub trait AudioResampler{
    fn new(original_frequency:u32, target_frequency:u32)->Self;
    fn resample(&mut self, buffer:&[StereoSample; BUFFER_SIZE])->Vec<StereoSample>;
}

pub trait ResampledAudioDevice<AR:AudioResampler> : AudioDevice{
    const VOLUME:Sample = 10 as Sample;

    fn push_buffer(&mut self, buffer:&[StereoSample; BUFFER_SIZE]){
        let resample = self.get_resampler().resample(buffer);
        for sample in resample{
            let(buffer, index) = self.get_audio_buffer();
            buffer[*index] = sample.left_sample * Self::VOLUME;
            buffer[*index + 1] = sample.left_sample * Self::VOLUME;
            *index += 2;
            if *index == BUFFER_SIZE{
                *index = 0;
                self.full_buffer_callback().unwrap();
            }
        }
    }

    fn get_audio_buffer(&mut self)->(&mut [Sample;BUFFER_SIZE], &mut usize);
    fn get_resampler(&mut self)->&mut AR;
    fn full_buffer_callback(&self)->Result<(), String>;
    fn new(frequency:i32, turbo_mul:u8)->Self;
}
