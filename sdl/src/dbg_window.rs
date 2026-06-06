use magenboy_common::gl_gfx_device::GlGfxDevice;
use magenboy_core::{debugger::{PpuLayer, PPU_BUFFER_SIZE, PPU_BUFFER_HEIGHT, PPU_BUFFER_WIDTH}, Pixel, utils::vec2::Vec2};

use sdl2::sys::*;

use crate::window::SdlWindow;

pub struct PpuLayerWindow{
    sdl_window: SdlWindow,
    renderer: GlGfxDevice,
}

impl PpuLayerWindow{
    pub fn new(renderer: GlGfxDevice, layer: PpuLayer)->Self{

        let layer_name = match layer{
            PpuLayer::Background => "Background",
            PpuLayer::Window => "Window",
            PpuLayer::Sprites => "Sprites"
        };

        let name = std::format!("Ppu {} debugger", layer_name);
        
        return Self { 
            sdl_window: SdlWindow::new(
                name,
                Vec2 { x: PPU_BUFFER_WIDTH, y: PPU_BUFFER_HEIGHT }, 
                1,
            ),
            renderer
        };
    }

    pub fn run(&mut self, buffer:&[Pixel; PPU_BUFFER_SIZE]){
        unsafe{
            SDL_RaiseWindow(self.sdl_window.window_handle);
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
