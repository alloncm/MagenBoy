use std::ffi::{CString, c_void};
use sdl2::sys::*;
use magenboy_core::{ppu::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, {Pixel, GfxDevice}, debugger::PpuLayer};
use crate::sdl::utils::get_sdl_error_message;

pub struct SdlGfxDevice{
    _window_name: CString,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,
    discard:u8,
    turbo_mul:u8,
}

const SDL_PIXEL_FORMAT:u32 = if cfg!(feature = "u16pixel"){SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGB565 as u32}else{SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGB888 as u32};

impl SdlGfxDevice{
    pub fn new(window_name:&str, screen_scale: usize, turbo_mul:u8, disable_vsync:bool, full_screen:bool)->Self{
        let cs_wnd_name = CString::new(window_name).unwrap();

        let (_window, renderer, texture): (*mut SDL_Window, *mut SDL_Renderer, *mut SDL_Texture) = unsafe{
            if SDL_Init(SDL_INIT_VIDEO) != 0{
                std::panic!("Init error: {}", get_sdl_error_message());
            }

            let window_flags = if full_screen{                
                // Hide cursor
                SDL_ShowCursor(0);
                SDL_WindowFlags::SDL_WINDOW_FULLSCREEN_DESKTOP as u32
            }
            else{
                SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32
            };

            let wind:*mut SDL_Window = SDL_CreateWindow(
                cs_wnd_name.as_ptr(),
                SDL_WINDOWPOS_UNDEFINED_MASK as i32, SDL_WINDOWPOS_UNDEFINED_MASK as i32,
                SCREEN_WIDTH as i32 * screen_scale as i32, SCREEN_HEIGHT as i32 * screen_scale as i32, window_flags);

            let mut render_flags = SDL_RendererFlags::SDL_RENDERER_ACCELERATED as u32;
            if !disable_vsync{
                render_flags |= SDL_RendererFlags::SDL_RENDERER_PRESENTVSYNC as u32;
            }

            let rend: *mut SDL_Renderer = SDL_CreateRenderer(wind, -1, render_flags);

            if SDL_RenderSetLogicalSize(rend, (SCREEN_WIDTH as u32) as i32, (SCREEN_HEIGHT as u32) as i32) != 0{
                std::panic!("Error while setting logical rendering\nError:{}", get_sdl_error_message());
            }
            
            let tex: *mut SDL_Texture = SDL_CreateTexture(rend, SDL_PIXEL_FORMAT,
                SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);

            SDL_SetWindowMinimumSize(wind, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);

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
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        self.discard = (self.discard + 1) % self.turbo_mul;
        if self.discard != 0{
            return;
        }

        unsafe{
            let mut pixels: *mut c_void = std::ptr::null_mut();
            let mut length: std::os::raw::c_int = 0;
            SDL_LockTexture(self.texture, std::ptr::null(), &mut pixels, &mut length);
            std::ptr::copy_nonoverlapping(buffer.as_ptr(),pixels as *mut Pixel,  buffer.len());
            SDL_UnlockTexture(self.texture);
            
            // Clear renderer cause the window can be resized
            SDL_RenderClear(self.renderer);
            SDL_RenderCopy(self.renderer, self.texture, std::ptr::null(), std::ptr::null());
            SDL_RenderPresent(self.renderer);
        }
    }
}

cfg_if::cfg_if!{ if #[cfg(feature = "dbg")]{
    pub struct PpuLayerWindow{
        _window_name: CString,
        renderer: *mut SDL_Renderer,
        texture: *mut SDL_Texture,
    }

    impl PpuLayerWindow{
        pub fn new(layer:PpuLayer)->Self{
            let layer_name = match layer{
                PpuLayer::Background => "Background",
                PpuLayer::Window => "Window",
                PpuLayer::Sprites => "Sprites"
            };
            let name = std::format!("Ppu {} debugger", layer_name);
            let c_name = CString::new(name).unwrap();
            unsafe{
                let window:*mut SDL_Window = SDL_CreateWindow(
                    c_name.as_ptr(),
                    SDL_WINDOWPOS_UNDEFINED_MASK as i32, SDL_WINDOWPOS_UNDEFINED_MASK as i32,
                    0x100, 0x100, 0);
                let renderer: *mut SDL_Renderer = SDL_CreateRenderer(window, -1, 0);
                let texture: *mut SDL_Texture = SDL_CreateTexture(renderer, SDL_PIXEL_FORMAT,
                    SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32, 0x100, 0x100);

                return PpuLayerWindow { _window_name: c_name, renderer, texture};
            }
        }

        pub fn render(&mut self, buffer:[Pixel;0x100*0x100]){
            unsafe{
                let mut pixels: *mut c_void = std::ptr::null_mut();
                let mut length: std::os::raw::c_int = 0;
                SDL_LockTexture(self.texture, std::ptr::null(), &mut pixels, &mut length);
                std::ptr::copy_nonoverlapping(buffer.as_ptr(),pixels as *mut Pixel,  buffer.len());
                SDL_UnlockTexture(self.texture);

                SDL_RenderCopy(self.renderer, self.texture, std::ptr::null(), std::ptr::null());
                SDL_RenderPresent(self.renderer);
            }
        }
    }
}}