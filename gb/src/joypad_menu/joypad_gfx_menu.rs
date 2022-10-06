use lib_gb::ppu::{gfx_device::*, gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, colors::{BLACK, WHITE}};

use super::{font::{self, GLYPH_HEIGHT, GLYPH_WIDTH}, MenuRenderer};

pub struct GfxDeviceMenuRenderer<'a, GFX:GfxDevice>{
    device:&'a mut GFX
}

impl<'a, GFX: GfxDevice> GfxDeviceMenuRenderer<'a, GFX> {
    pub fn new(device: &'a mut GFX) -> Self { Self { device } }
}

impl<'a, GFX: GfxDevice, T> MenuRenderer<T> for GfxDeviceMenuRenderer<'a, GFX>{
    fn render_menu(&mut self, menu:&Vec<super::MenuOption<T>>, selection:usize) {
        let mut frame_buffer = [0 as Pixel; SCREEN_HEIGHT * SCREEN_WIDTH];
        let mut frame_buffer_height_index = 0;
        for option_index in 0..menu.len(){
            let prompt = &menu[option_index].prompt;
            let mut width_index = 0;
            for char in prompt.as_bytes(){
                // TODO: add support for non alphabetical chars
                if !char.is_ascii_alphabetic(){continue;}
                // TODO: add support for lower case alphabetical chars
                let char = char.to_ascii_uppercase();
                let glyph = font::FONT_LUT[(char - 'A' as u8) as usize];
                for i in 0..GLYPH_HEIGHT{
                    for j in 0..GLYPH_WIDTH{
                        let pixel = if selection == option_index{
                            Pixel::from(if glyph[i * GLYPH_WIDTH + j] {BLACK}else{WHITE})
                        }
                        else{
                            Pixel::from(if glyph[i * GLYPH_WIDTH + j] {WHITE}else{BLACK})
                        };
                        frame_buffer[(frame_buffer_height_index + i) * SCREEN_WIDTH + width_index + j] = pixel;
                    }
                }
                width_index += GLYPH_WIDTH;
            }
            frame_buffer_height_index += GLYPH_HEIGHT;
        }
        self.device.swap_buffer(&frame_buffer);
    }
}