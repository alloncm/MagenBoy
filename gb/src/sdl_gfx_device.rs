use std::ffi::{CString, c_void};

use lib_gb::ppu::{gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::GfxDevice};
use sdl2::sys::*;

pub struct SdlGfxDevice{
    _window_name: CString,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,
    width:u32,
    height:u32,
    sacle:u8,
    frame_start_time:u64,
    frame_time_ms:f64,
    discard:u8,
    turbo_mul:u8,
}

impl SdlGfxDevice{
    pub fn new(buffer_width:u32, buffer_height:u32, window_name:&str, screen_scale: u8, turbo_mul:u8)->Self{
        let cs_wnd_name = CString::new(window_name).unwrap();

        let (_window, renderer, texture): (*mut SDL_Window, *mut SDL_Renderer, *mut SDL_Texture) = unsafe{
            SDL_Init(SDL_INIT_VIDEO);
            let wind:*mut SDL_Window = SDL_CreateWindow(
                cs_wnd_name.as_ptr(),
                SDL_WINDOWPOS_UNDEFINED_MASK as i32, SDL_WINDOWPOS_UNDEFINED_MASK as i32,
                buffer_width as i32 * screen_scale as i32, buffer_height as i32 * screen_scale as i32, 0);
            
            let rend: *mut SDL_Renderer = SDL_CreateRenderer(wind, -1, 0);
            
            let tex: *mut SDL_Texture = SDL_CreateTexture(rend,
                SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32, SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32,
                    buffer_width as i32 * screen_scale as i32, buffer_height as i32 * screen_scale as i32);
            
            (wind, rend, tex)
        };
        
        Self{
            _window_name: cs_wnd_name,
            renderer,
            texture,
            height:buffer_height,
            width:buffer_width,
            sacle:screen_scale,
            frame_start_time: unsafe{SDL_GetPerformanceCounter()},
            frame_time_ms: (1.0/(60.0 as f64)) * 1_000.0,
            discard:0,
            turbo_mul
        }
    }

    fn extend_vec(vec:&[u32], scale:usize, w:usize, h:usize)->Vec<u32>{
        let mut new_vec = vec![0;vec.len()*scale*scale];
        for y in 0..h{
            let sy = y*scale;
            for x in 0..w{
                let sx = x*scale;
                for i in 0..scale{
                    for j in 0..scale{
                        new_vec[(sy+i)*(w*scale)+sx+j] = vec[y*w+x];
                    }
                }
            } 
        }
        return new_vec;
    }
}

impl GfxDevice for SdlGfxDevice{
    fn swap_buffer(&mut self, buffer:&[u32; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        self.discard = (self.discard + 1) % self.turbo_mul;
        if self.discard != 0{
            return;
        }

        unsafe{
            let extended_buffer = Self::extend_vec(buffer, self.sacle as usize, self.width as usize, self.height as usize);

            let mut pixels: *mut c_void = std::ptr::null_mut();
            let mut length: std::os::raw::c_int = 0;
            SDL_LockTexture(self.texture, std::ptr::null(), &mut pixels, &mut length);
            std::ptr::copy_nonoverlapping(extended_buffer.as_ptr(),pixels as *mut u32,  extended_buffer.len());
            SDL_UnlockTexture(self.texture);
            
            //There is no need to call SDL_RenderClear since im replacing the whole buffer 
            SDL_RenderCopy(self.renderer, self.texture, std::ptr::null(), std::ptr::null());
            SDL_RenderPresent(self.renderer);

            let frame_end_time = SDL_GetPerformanceCounter();
            let elapsed = ((frame_end_time - self.frame_start_time) as f64 / (SDL_GetPerformanceFrequency() as f64)) * 1_000.0;
            if elapsed < self.frame_time_ms{
                // SDL_Delay(((self.frame_time_ms - elapsed).floor()) as Uint32);
            }

            self.frame_start_time = SDL_GetPerformanceCounter();
        }
    }
}