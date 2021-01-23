use std::{
    vec::Vec,
    mem::MaybeUninit,
    ffi::{CStr, c_void}
};
use lib_gb::apu::audio_device::AudioDevice;
use sdl2::{
    sys::*,
    libc::c_char
};

const GB_SOUND_FREQUENCY:u32 = 4_194_304;
const BUFFER_SIZE:usize = 1024;

pub struct SdlAudioDevie{
    device_id: SDL_AudioDeviceID,
    to_skip:u32,

    buffer: Vec<f32>
}

impl SdlAudioDevie{
    pub fn new(frequency:i32, channels:u8)->Self{
        let to_skip = GB_SOUND_FREQUENCY / frequency as u32;
        if to_skip == 0{
            std::panic!("freqency is too high: {}", frequency);
        }

        let desired_audio_spec = SDL_AudioSpec{
            freq: frequency,
            format: AUDIO_F32SYS as u16,
            channels: channels,
            silence: 0,
            samples: BUFFER_SIZE as u16,
            padding: 0,
            size: 0,
            callback: Option::None,
            userdata: std::ptr::null_mut()
        };

        
        let mut uninit_audio_spec:MaybeUninit<SDL_AudioSpec> = MaybeUninit::uninit();

        let device_id = unsafe{
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
            to_skip:to_skip,
            buffer:Vec::with_capacity(BUFFER_SIZE)
        };
    }

    fn get_sdl_error_message()->&'static str{
        unsafe{
            let error_message:*const c_char = SDL_GetError();
            
            return CStr::from_ptr(error_message).to_str().unwrap();
        }
    }


    fn push_audio_to_device(&self, audio:&[f32])->Result<(),&str>{
        let audio_ptr: *const c_void = audio.as_ptr() as *const c_void;
        let data_byte_len = (audio.len() * std::mem::size_of::<f32>()) as u32;

        unsafe{
            SDL_ClearError();
            if SDL_QueueAudio(self.device_id, audio_ptr, data_byte_len) != 0{
                return Err(Self::get_sdl_error_message());
            }
            
            Ok(())
        }
    }
}

impl AudioDevice for SdlAudioDevie{
    fn push_buffer(&mut self, buffer:&[f32]){
        let mut counter = 0;
        for sample in buffer.into_iter(){
            if *sample != 0.0{
                //log::info!("{}", sample)
            }
            
            if counter == self.to_skip{
                self.buffer.push(*sample);
                counter = 0;

                if self.buffer.len() == BUFFER_SIZE{
                    self.push_audio_to_device(&self.buffer).unwrap();
                    self.buffer.clear();
                }
            }
            else{
                counter += 1;
            }
        }
    }
}