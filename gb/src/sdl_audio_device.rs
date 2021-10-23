use std::{vec::Vec,mem::MaybeUninit,ffi::{CStr, c_void}};
use lib_gb::{GB_FREQUENCY, apu::audio_device::*};
use sdl2::{sys::*,libc::c_char};
use crate::audio_resampler::AudioResampler;

//After twicking those numbers Iv reached this, this will affect fps which will affect sound tearing
const BUFFER_SIZE:usize = 1024 * 2;
const BYTES_TO_WAIT:u32 = BUFFER_SIZE as u32 * 16;
const VOLUME:Sample = 10 as Sample;

pub struct SdlAudioDevie{
    device_id: SDL_AudioDeviceID,
    resampler: AudioResampler,

    buffer: Vec<Sample>
}

impl SdlAudioDevie{
    pub fn new(frequency:i32, turbo_mul:u8)->Self{

        let desired_audio_spec = SDL_AudioSpec{
            freq: frequency,
            format: AUDIO_S16SYS as u16,
            channels: 2,
            silence: 0,
            samples: BUFFER_SIZE as u16,
            padding: 0,
            size: 0,
            callback: Option::None,
            userdata: std::ptr::null_mut()
        };

        
        let mut uninit_audio_spec:MaybeUninit<SDL_AudioSpec> = MaybeUninit::uninit();

        let device_id = unsafe{
            SDL_Init(SDL_INIT_AUDIO);
            SDL_ClearError();
            let id = SDL_OpenAudioDevice(std::ptr::null(), 0, &desired_audio_spec, uninit_audio_spec.as_mut_ptr() , 0);

            if id == 0{
                std::panic!("{}",Self::get_sdl_error_message());
            }

            let init_audio_spec:SDL_AudioSpec = uninit_audio_spec.assume_init();

            if init_audio_spec.freq != frequency {
                std::panic!("Error initializing audio could not use the frequency: {}", frequency);
            }

            //This will start the audio processing
            SDL_PauseAudioDevice(id, 0);

            id
        };
        
        return SdlAudioDevie{
            device_id: device_id,
            buffer:Vec::with_capacity(BUFFER_SIZE),
            resampler: AudioResampler::new(GB_FREQUENCY * turbo_mul as u32, frequency as u32)
        };
    }

    fn get_sdl_error_message()->&'static str{
        unsafe{
            let error_message:*const c_char = SDL_GetError();
            
            return CStr::from_ptr(error_message).to_str().unwrap();
        }
    }


    fn push_audio_to_device(&self, audio:&[Sample])->Result<(),&str>{
        let audio_ptr: *const c_void = audio.as_ptr() as *const c_void;
        let data_byte_len = (audio.len() * std::mem::size_of::<Sample>()) as u32;

        unsafe{
            while SDL_GetQueuedAudioSize(self.device_id) > BYTES_TO_WAIT{
                SDL_Delay(1);
            }

            SDL_ClearError();
            if SDL_QueueAudio(self.device_id, audio_ptr, data_byte_len) != 0{
                return Err(Self::get_sdl_error_message());
            }
            
            Ok(())
        }
    }
}

impl AudioDevice for SdlAudioDevie{
    fn push_buffer(&mut self, buffer:&[StereoSample]){
        for sample in self.resampler.resample(buffer){

            self.buffer.push(sample.left_sample * VOLUME);
            self.buffer.push(sample.right_sample * VOLUME);

            if self.buffer.len() == BUFFER_SIZE{
                self.push_audio_to_device(&self.buffer).unwrap();
                self.buffer.clear();
            }
        }
    }
}