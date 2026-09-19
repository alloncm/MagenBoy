use std::mem::{MaybeUninit, size_of};

use sdl2::sys::*;

use magenboy_core::{GB_FREQUENCY, apu::audio_device::*};
use magenboy_common::audio::AudioResampler;

use crate::utils::get_sdl_error_message;

const BUFFER_SIZE: usize = 512;

pub struct SdlAudioDevice<AR:AudioResampler>{
    resampler: AR,
    device_id:SDL_AudioDeviceID,

    buffer: [StereoSample; BUFFER_SIZE],
    buffer_index: usize,
    frequency: u32,
}

impl<AR:AudioResampler> SdlAudioDevice<AR>{
    pub fn new(frequency:i32, turbo_mul:u8)->Self{
        let mut device = SdlAudioDevice{
            resampler: AudioResampler::new(GB_FREQUENCY * turbo_mul as u32, frequency as u32),
            device_id:0,
            buffer: [StereoSample::const_defualt(); BUFFER_SIZE],
            buffer_index: 0,
            frequency: frequency as u32,    // Panic if the desired freq is not avaliable
        };
        
        let desired_audio_spec = SDL_AudioSpec{
            freq: frequency,
            format: AUDIO_S16SYS as u16,    // assumes Sample type is i16
            channels: 2,
            silence: 0,
            samples: BUFFER_SIZE as u16,
            padding: 0,
            size: 0,
            callback: Option::None,
            userdata: std::ptr::null_mut()
        };

        unsafe{
            SDL_ClearError();
            let mut uninit_audio_spec:MaybeUninit<SDL_AudioSpec> = MaybeUninit::uninit();
            let id = SDL_OpenAudioDevice(std::ptr::null(), 0, &desired_audio_spec, uninit_audio_spec.as_mut_ptr() , 0);

            if id == 0{
                std::panic!("{}", get_sdl_error_message());
            }

            let init_audio_spec:SDL_AudioSpec = uninit_audio_spec.assume_init();

            if init_audio_spec.freq != desired_audio_spec.freq {
                std::panic!("Error initializing audio could not use the frequency: {}", desired_audio_spec.freq);
            }

            //This will start the audio processing
            SDL_PauseAudioDevice(id, 0);
            device.device_id = id;
        }

        return device;
    }

    fn get_queue_len(&self) -> usize {
        unsafe {
            SDL_GetQueuedAudioSize(self.device_id) as usize / size_of::<StereoSample>()
        }
    }
}

impl<AR:AudioResampler> AudioDevice for SdlAudioDevice<AR>{
    fn push_sample(&mut self, sample:StereoSample) {
        let Some(sample) = self.resampler.resample(sample) else {
            return;
        };

        // Max lag of 0.125 seconds in audio vs video
        if self.get_queue_len() > (self.frequency / 8) as usize {
            return;
        }
    
        self.buffer[self.buffer_index] = sample;
        self.buffer_index += 1;
        if self.buffer_index == BUFFER_SIZE {
            self.buffer_index = 0;
            unsafe{
                SDL_QueueAudio(self.device_id, self.buffer.as_ptr() as _, (self.buffer.len() * size_of::<StereoSample>()) as u32);
            }
        }
    }
}

impl<AR:AudioResampler> Drop for SdlAudioDevice<AR>{
    fn drop(&mut self) {
        unsafe{
            SDL_CloseAudioDevice(self.device_id);
        }
    }
}
