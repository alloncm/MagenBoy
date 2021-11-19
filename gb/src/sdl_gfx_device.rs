use std::ffi::{CString, c_void};

use lib_gb::ppu::{gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::GfxDevice};
use sdl2::sys::*;

pub struct SdlGfxDevice{
    _window_name: CString,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,
    discard:u8,
    turbo_mul:u8,
}

impl SdlGfxDevice{
    pub fn new(window_name:&str, screen_scale: u8, turbo_mul:u8, disable_vsync:bool, full_screen:bool)->Self{
        let cs_wnd_name = CString::new(window_name).unwrap();

        let (_window, renderer, texture): (*mut SDL_Window, *mut SDL_Renderer, *mut SDL_Texture) = unsafe{
            SDL_Init(SDL_INIT_VIDEO);

            let window_flags = if full_screen{
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
            
            if SDL_RenderSetLogicalSize(rend, (SCREEN_WIDTH as u32) as i32, (SCREEN_HEIGHT as u32) as i32) != 0{
                std::panic!("Error while setting logical rendering");
            }

            let tex: *mut SDL_Texture = SDL_CreateTexture(rend,
                SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32, SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32,
                    SCREEN_WIDTH as i32 , SCREEN_HEIGHT as i32 );
            
            (wind, rend, tex)
        };
        
        Self{
            _window_name: cs_wnd_name,
            renderer,
            texture,
            discard:0,
            turbo_mul
        }
    }
}

impl GfxDevice for SdlGfxDevice{
    fn swap_buffer(&mut self, buffer:&[u32; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        self.discard = (self.discard + 1) % self.turbo_mul;
        if self.discard != 0{
            return;
        }

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