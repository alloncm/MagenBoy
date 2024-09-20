use std::ffi::CStr;

use sdl2::{libc::c_char, sys::*};

pub fn get_sdl_error_message()->&'static str{
    unsafe{
        let error_message:*const c_char = SDL_GetError();
        
        return CStr::from_ptr(error_message).to_str().unwrap();
    }
}