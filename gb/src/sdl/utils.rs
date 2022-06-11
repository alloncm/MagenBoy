use std::{ffi::CStr, mem::MaybeUninit};
use sdl2::{libc::c_char, sys::*};

pub fn get_sdl_error_message()->&'static str{
    unsafe{
        let error_message:*const c_char = SDL_GetError();
        
        return CStr::from_ptr(error_message).to_str().unwrap();
    }
}

pub fn init_sdl_audio_device(audio_spec:&SDL_AudioSpec)->SDL_AudioDeviceID{
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