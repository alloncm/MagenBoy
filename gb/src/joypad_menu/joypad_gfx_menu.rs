use lib_gb::ppu::{gfx_device::*, gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, colors::{BLACK, WHITE}};

use super::{font::*, MenuRenderer};

pub struct GfxDeviceMenuRenderer<'a, GFX:GfxDevice>{
    device:&'a mut GFX
}

impl<'a, GFX: GfxDevice> GfxDeviceMenuRenderer<'a, GFX> {
    pub fn new(device: &'a mut GFX) -> Self { Self { device } }
}

impl<'a, GFX: GfxDevice, T> MenuRenderer<T> for GfxDeviceMenuRenderer<'a, GFX>{
    fn render_menu(&mut self, menu:&Vec<super::MenuOption<T>>, selection:usize) {
        let mut frame_buffer = [0 as Pixel; SCREEN_HEIGHT * SCREEN_WIDTH];

        // Calculate the range of the visible menu
        let mut start_index = 0;
        let mut end_index = std::cmp::min(menu.len(), SCREEN_HEIGHT / GLYPH_HEIGHT);
        if selection >= end_index{
            end_index = selection + 1;
            start_index = end_index - (SCREEN_HEIGHT / GLYPH_HEIGHT);
        }

        let mut frame_buffer_height_index = 0;
        for option_index in start_index..end_index{
            let prompt = &menu[option_index].prompt;
            let mut width_index = 0;
            for char in prompt.as_bytes(){
                let glyph = FONT_LUT[(char - FONT_ASCII_START_INDEX) as usize];
                for i in 0..GLYPH_HEIGHT{
                    for j in 0..GLYPH_WIDTH{
                        let color = if selection == option_index{
                            if glyph[i * GLYPH_WIDTH + j] {BLACK}else{WHITE}
                        }
                        else{
                            if glyph[i * GLYPH_WIDTH + j] {WHITE}else{BLACK}
                        };
                        frame_buffer[(frame_buffer_height_index + i) * SCREEN_WIDTH + width_index + j] = Pixel::from(color);
                    }
                }
                width_index += GLYPH_WIDTH;

                // if the game name it too long cut it 
                if width_index >= SCREEN_WIDTH{
                    break;
                }
            }
            frame_buffer_height_index += GLYPH_HEIGHT;
        }
        self.device.swap_buffer(&frame_buffer);
    }
}