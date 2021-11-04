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

use std::{ffi::CStr, mem::MaybeUninit};
use lib_gb::apu::audio_device::{AudioDevice, BUFFER_SIZE, Sample, StereoSample};
use sdl2::{libc::c_char, sys::*};

fn get_sdl_error_message()->&'static str{
    unsafe{
        let error_message:*const c_char = SDL_GetError();
        
        return CStr::from_ptr(error_message).to_str().unwrap();
    }
}

fn init_sdl_audio_device(audio_spec:&SDL_AudioSpec)->SDL_AudioDeviceID{
    let mut uninit_audio_spec:MaybeUninit<SDL_AudioSpec> = MaybeUninit::uninit();

    unsafe{
        SDL_Init(SDL_INIT_AUDIO);
        SDL_ClearError();
        let id = SDL_OpenAudioDevice(std::ptr::null(), 0, audio_spec, uninit_audio_spec.as_mut_ptr() , 0);

        if id == 0{
            std::panic!("{}", get_sdl_error_message());
        }

        let init_audio_spec:SDL_AudioSpec = uninit_audio_spec.assume_init();

        if init_audio_spec.freq != audio_spec.freq {
            std::panic!("Error initializing audio could not use the frequency: {}", audio_spec.freq);
        }

        //This will start the audio processing
        SDL_PauseAudioDevice(id, 0);
        return id;
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
