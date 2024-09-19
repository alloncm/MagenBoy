use std::ffi::{CString, c_void};
use sdl2::sys::*;
use magenboy_core::{ppu::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, utils::vec2::Vec2, GfxDevice, Pixel};
use crate::sdl::utils::get_sdl_error_message;

const SDL_PIXEL_FORMAT:u32 = if cfg!(feature = "u16pixel"){SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGB565 as u32}else{SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGB888 as u32};

struct SdlWindow{
    _window_name: CString,
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,
}

impl SdlWindow{
    fn new(window_name:&str, dimensions: Vec2<usize>, screen_scale: usize, disable_vsync:bool, window_flags:u32)->Self{
        let cs_wnd_name = CString::new(window_name).unwrap();
        let width = dimensions.x as i32;
        let height = dimensions.y as i32;
        unsafe{
            if SDL_Init(SDL_INIT_VIDEO) != 0{
                std::panic!("Init error: {}", get_sdl_error_message());
            }

            let window:*mut SDL_Window = SDL_CreateWindow(
                cs_wnd_name.as_ptr(),SDL_WINDOWPOS_UNDEFINED_MASK as i32, SDL_WINDOWPOS_UNDEFINED_MASK as i32,
                 width * screen_scale as i32, height * screen_scale as i32, window_flags);

            let mut render_flags = SDL_RendererFlags::SDL_RENDERER_ACCELERATED as u32;
            if !disable_vsync{
                render_flags |= SDL_RendererFlags::SDL_RENDERER_PRESENTVSYNC as u32;
            }

            let renderer: *mut SDL_Renderer = SDL_CreateRenderer(window, -1, render_flags);

            if SDL_RenderSetLogicalSize(renderer, width , height) != 0{
                std::panic!("Error while setting logical rendering\nError:{}", get_sdl_error_message());
            }
            
            let texture: *mut SDL_Texture = SDL_CreateTexture(renderer, SDL_PIXEL_FORMAT,SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32, width, height);

            SDL_SetWindowMinimumSize(window, width, height);
        
            return Self{_window_name: cs_wnd_name, window, renderer, texture};
        }
    }

    fn render(&self, buffer: &[Pixel]) {
        unsafe{
            let mut pixels: *mut c_void = std::ptr::null_mut();
            let mut length: std::os::raw::c_int = 0;
            SDL_LockTexture(self.texture, std::ptr::null(), &mut pixels, &mut length);
            std::ptr::copy_nonoverlapping(buffer.as_ptr(),pixels as *mut Pixel,  buffer.len());
            SDL_UnlockTexture(self.texture);
    
            // Clear renderer for cases where the window could be resized
            SDL_RenderClear(self.renderer);
            SDL_RenderCopy(self.renderer, self.texture, std::ptr::null(), std::ptr::null());
            SDL_RenderPresent(self.renderer);
        }
    }
}

impl Drop for SdlWindow{
    fn drop(&mut self) {
        unsafe{
            SDL_DestroyTexture(self.texture);
            SDL_DestroyRenderer(self.renderer);
            SDL_DestroyWindow(self.window);
        }
    }
}

pub struct SdlGfxDevice{
    sdl_window:SdlWindow,
    discard:u8,
    turbo_mul:u8,
}

impl SdlGfxDevice{
    pub fn new(window_name:&str, screen_scale: usize, turbo_mul:u8, disable_vsync:bool, full_screen:bool)->Self{
        
        let window_flags = if full_screen{                
            // Hide cursor
            unsafe{SDL_ShowCursor(0);}
            SDL_WindowFlags::SDL_WINDOW_FULLSCREEN_DESKTOP as u32
        }
        else{
            SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32
        };
        
        return Self{discard:0, turbo_mul, sdl_window: SdlWindow::new(window_name, Vec2{x:SCREEN_WIDTH, y:SCREEN_HEIGHT}, screen_scale, disable_vsync, window_flags)};
    }

    pub fn poll_event(&self)->Option<SDL_Event>{
        unsafe{
            let mut event: std::mem::MaybeUninit<SDL_Event> = std::mem::MaybeUninit::uninit();
            // updating the events for the whole app
            SDL_PumpEvents();
            if SDL_PollEvent(event.as_mut_ptr()) != 0{
                return Option::Some(event.assume_init());
            }
            return Option::None;
        }
    }
}

impl GfxDevice for SdlGfxDevice{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        self.discard = (self.discard + 1) % self.turbo_mul;
        if self.discard != 0{
            return;
        }
        self.sdl_window.render(buffer);
    }
}

#[cfg(feature = "dbg")]
pub struct PpuLayerWindow{
    sdl_window: SdlWindow
}

#[cfg(feature = "dbg")]
impl PpuLayerWindow{
    pub fn new(layer:magenboy_core::debugger::PpuLayer)->Self{
        use magenboy_core::debugger::{PpuLayer, PPU_BUFFER_HEIGHT, PPU_BUFFER_WIDTH};

        let layer_name = match layer{
            PpuLayer::Background => "Background",
            PpuLayer::Window => "Window",
            PpuLayer::Sprites => "Sprites"
        };

        let name = std::format!("Ppu {} debugger", layer_name);
        
        let window_flags = SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32 | SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32;
        return Self { sdl_window: SdlWindow::new(&name, Vec2 { x: PPU_BUFFER_WIDTH, y: PPU_BUFFER_HEIGHT }, 1, false, window_flags)};
    }

    pub fn run(&mut self, buffer:&[Pixel;magenboy_core::debugger::PPU_BUFFER_SIZE]){
        unsafe{
            SDL_RaiseWindow(self.sdl_window.window);
            let mut event: std::mem::MaybeUninit<SDL_Event> = std::mem::MaybeUninit::uninit();
            loop{
                self.sdl_window.render(buffer);
                SDL_PumpEvents();
                if SDL_PollEvent(event.as_mut_ptr()) != 0{
                    let event: SDL_Event = event.assume_init();
                    if event.type_ == SDL_EventType::SDL_WINDOWEVENT as u32 && event.window.event == SDL_WindowEventID::SDL_WINDOWEVENT_CLOSE as u8{
                        break;
                    }
                }
            }   
        }
    }
}
