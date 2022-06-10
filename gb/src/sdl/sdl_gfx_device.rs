use std::ffi::{CString, c_void};
use sdl2::sys::*;
use lib_gb::ppu::{gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::{GfxDevice, Pixel}};
use crate::sdl::utils::get_sdl_error_message;

pub struct SdlGfxDevice{
    _window_name: CString,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,
    discard:u8,
    turbo_mul:u8,
    #[cfg(feature = "static-scale")]
    screen_scale:usize,
}

impl SdlGfxDevice{
    pub fn new(window_name:&str, screen_scale: usize, turbo_mul:u8, disable_vsync:bool, full_screen:bool)->Self{
        #[cfg(feature = "u16pixel")]
        std::compile_error("Sdl gfx device must have Pixel type = u32");

        let cs_wnd_name = CString::new(window_name).unwrap();

        let (_window, renderer, texture): (*mut SDL_Window, *mut SDL_Renderer, *mut SDL_Texture) = unsafe{
            if SDL_Init(SDL_INIT_VIDEO) != 0{
                std::panic!("Init error: {}", get_sdl_error_message());
            }

            let window_flags = if full_screen{
                #[cfg(feature = "static-scale")]
                log::warn!("Please notice that this binary have been compiled with the static-scale feature and you are running with the full screen option.\nThe rendering window might be in wrong scale.");
                
                // Hide cursor
                SDL_ShowCursor(0);
                SDL_WindowFlags::SDL_WINDOW_FULLSCREEN_DESKTOP as u32
            }
            else{
                0
            };

            let wind:*mut SDL_Window = SDL_CreateWindow(
                cs_wnd_name.as_ptr(),
                SDL_WINDOWPOS_UNDEFINED_MASK as i32, SDL_WINDOWPOS_UNDEFINED_MASK as i32,
                SCREEN_WIDTH as i32 * screen_scale as i32, SCREEN_HEIGHT as i32 * screen_scale as i32,
                 window_flags);

            let mut render_flags = SDL_RendererFlags::SDL_RENDERER_ACCELERATED as u32;
            if !disable_vsync{
                render_flags |= SDL_RendererFlags::SDL_RENDERER_PRESENTVSYNC as u32;
            }

            let rend: *mut SDL_Renderer = SDL_CreateRenderer(wind, -1, render_flags);
            
            let texture_width:i32;
            let texture_height:i32;

            cfg_if::cfg_if!{
                if #[cfg(feature = "static-scale")]{
                    texture_height = SCREEN_HEIGHT as i32 * screen_scale as i32;
                    texture_width = SCREEN_WIDTH as i32 * screen_scale as i32;
                }
                else{
                    if SDL_RenderSetLogicalSize(rend, (SCREEN_WIDTH as u32) as i32, (SCREEN_HEIGHT as u32) as i32) != 0{
                        std::panic!("Error while setting logical rendering\nError:{}", get_sdl_error_message());
                    }
                    texture_height = SCREEN_HEIGHT as i32;
                    texture_width = SCREEN_WIDTH as i32;
                }
            }
            
            let tex: *mut SDL_Texture = SDL_CreateTexture(rend,
                SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGB888 as u32, SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32,
                    texture_width, texture_height);
            
            (wind, rend, tex)
        };
        
        Self{
            _window_name: cs_wnd_name,
            renderer,
            texture,
            discard:0,
            turbo_mul, 
            #[cfg(feature = "static-scale")]
            screen_scale
        }
    }

    #[cfg(feature = "static-scale")]
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
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        self.discard = (self.discard + 1) % self.turbo_mul;
        if self.discard != 0{
            return;
        }

        #[cfg(feature = "static-scale")]
        let buffer = Self::extend_vec(buffer, self.screen_scale, SCREEN_WIDTH, SCREEN_HEIGHT);

        unsafe{
            let mut pixels: *mut c_void = std::ptr::null_mut();
            let mut length: std::os::raw::c_int = 0;
            SDL_LockTexture(self.texture, std::ptr::null(), &mut pixels, &mut length);
            std::ptr::copy_nonoverlapping(buffer.as_ptr(),pixels as *mut u32,  buffer.len());
            SDL_UnlockTexture(self.texture);
            
            //There is no need to call SDL_RenderClear since im replacing the whole buffer 
            SDL_RenderCopy(self.renderer, self.texture, std::ptr::null(), std::ptr::null());
            SDL_RenderPresent(self.renderer);
        }
    }
}