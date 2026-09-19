use magenboy_core::{debugger::{PpuLayer, PPU_BUFFER_SIZE, PPU_BUFFER_HEIGHT, PPU_BUFFER_WIDTH}, Pixel, utils::vec2::Vec2};

use sdl2::sys::*;

use crate::{sdl_window::SdlWindow, sdl_gfx_device::SdlGfxDevice};

pub struct PpuLayerWindow{
    sdl_window: SdlWindow,
    renderer: SdlGfxDevice,
}

impl PpuLayerWindow{
    pub fn new(layer: PpuLayer)->Self{

        let layer_name = match layer{
            PpuLayer::Background => "Background",
            PpuLayer::Window => "Window",
            PpuLayer::Sprites => "Sprites"
        };

        let name = std::format!("Ppu {} debugger", layer_name);

        let window = SdlWindow::new(
            name,
            Vec2 { x: PPU_BUFFER_WIDTH, y: PPU_BUFFER_HEIGHT }, 
            1,
        );
        let renderer = SdlGfxDevice::new(window.get_sdl_window_handle());
        
        return Self { 
            sdl_window: window,
            renderer
        };
    }

    pub fn run(&mut self, buffer:&[Pixel; PPU_BUFFER_SIZE]){
        unsafe{
            SDL_RaiseWindow(self.sdl_window.get_sdl_window_handle().as_ptr());
            let mut event: std::mem::MaybeUninit<SDL_Event> = std::mem::MaybeUninit::uninit();
            loop{
                self.renderer.render(buffer, PPU_BUFFER_WIDTH as _, PPU_BUFFER_HEIGHT as _);
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
