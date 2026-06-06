use std::{ptr::null_mut, ffi::CString};

use magenboy_core::utils::vec2::Vec2;

use sdl2::sys::*;

use crate::utils::get_sdl_error_message;

pub struct SdlWindow {
    pub(crate) window_handle: *mut SDL_Window,
    gl_context: SDL_GLContext,
}

impl SdlWindow {
    pub fn new(name: String, dimensions: Vec2<usize>, screen_scale: usize) -> Self{
        let (sdl_window, sdl_gl_context) = unsafe {
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_CONTEXT_MAJOR_VERSION, 3);
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_CONTEXT_MINOR_VERSION, 3);
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_CONTEXT_PROFILE_MASK, SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_CORE as i32);

            let title = CString::new(name).unwrap();

            let window: *mut SDL_Window = SDL_CreateWindow(
                title.as_ptr() as _,
                SDL_WINDOWPOS_UNDEFINED_MASK as i32, 
                SDL_WINDOWPOS_UNDEFINED_MASK as i32,
                (dimensions.x * screen_scale) as i32,
                (dimensions.y * screen_scale) as i32, 
                SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32 | SDL_WindowFlags::SDL_WINDOW_OPENGL as u32
            );

            if window == null_mut() {
                std::panic!("Failed to create SDL window, message: {}", get_sdl_error_message());
            }

            let gl_context: SDL_GLContext = SDL_GL_CreateContext(window);
            if gl_context == null_mut() {
                std::panic!("Failed to get SDL GL context: message: {}", get_sdl_error_message());
            }
            // Enables vsync
            SDL_GL_SetSwapInterval(1);

            (window, gl_context)
        };

        return Self{
            gl_context: sdl_gl_context,
            window_handle: sdl_window,
        };
    }
}

impl Drop for SdlWindow {
    fn drop(&mut self) {
        unsafe{
            SDL_GL_DeleteContext(self.gl_context);
            SDL_DestroyWindow(self.window_handle);
        }
    }
}