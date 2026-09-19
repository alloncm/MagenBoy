use std::{ffi::CString, time::Instant, ptr};

use sdl2::sys::*;

use magenboy_common::gl_gfx_device::GlGfxDevice;
use magenboy_core::Pixel;

pub struct SdlGfxDevice {
    gl_gfx_device: GlGfxDevice,
    sdl_window_handle: ptr::NonNull<SDL_Window>,
    frames_counter: u32,
    timer: Instant
}

impl SdlGfxDevice {
    pub fn new(window_handle: ptr::NonNull<SDL_Window>) -> Self{
        let mut width: i32 = 0;
        let mut height: i32 = 0;
        // SAFETY: SDL call
        // window parameter is not null
        unsafe {
            // Enables vsync
            SDL_GL_SetSwapInterval(1);
            SDL_GetWindowSize(window_handle.as_ptr(), &mut width, &mut height);
        }

        let gl_gfx_device = GlGfxDevice::new(width as u32, height as u32, |s|{
            let name = CString::new(s).unwrap();
            unsafe{SDL_GL_GetProcAddress(name.as_ptr())}
        });

        Self { gl_gfx_device, sdl_window_handle: window_handle, frames_counter: 0, timer: Instant::now() }
    }

    pub fn render(&mut self, buffer:&[Pixel], width: u32, height: u32) {
        self.gl_gfx_device.render(buffer, width, height);
        
        // SAFETY: SDL call
        // window parameter is not null
        unsafe{SDL_GL_SwapWindow(self.sdl_window_handle.as_ptr())};

        // measure fps
        self.frames_counter += 1;
        let duration = self.timer.elapsed();
        
        if duration.as_millis() > 1000{
            log::debug!("FPS: {}", self.frames_counter);
            self.frames_counter = 0;
            self.timer = Instant::now();
        }
    }

    pub fn update_viewport(&mut self) {
        let mut width: i32 = 0;
        let mut height: i32 = 0;
        // SAFETY: params are initialized
        unsafe{SDL_GetWindowSize(self.sdl_window_handle.as_ptr(), &mut width, &mut height)};

        GlGfxDevice::update_viewport(width, height);
    }
}
