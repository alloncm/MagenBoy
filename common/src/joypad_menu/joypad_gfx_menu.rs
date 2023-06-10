use lib_gb::ppu::{gfx_device::*, gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, color::{BLACK, WHITE}, color::Color};

use super::{font::*, MenuRenderer};

const ORANGE:Color = Color{r:0xFF, g:0xA5, b:0x0};

const HEADER_COLOR:Color = ORANGE;
const BACKGROUND_COLOR:Color = BLACK;
const TEXT_COLOR:Color = WHITE;

pub struct GfxDeviceMenuRenderer<'a, GFX:GfxDevice>{
    device:&'a mut GFX
}

impl<'a, GFX: GfxDevice> GfxDeviceMenuRenderer<'a, GFX> {
    pub fn new(device: &'a mut GFX) -> Self { Self { device } }
}

impl<'a, GFX: GfxDevice, T, S:AsRef<str>> MenuRenderer<T, S> for GfxDeviceMenuRenderer<'a, GFX>{
    fn render_menu(&mut self, header:&S, menu:&[super::MenuOption<T, S>], selection:usize) {
        let mut frame_buffer = [0 as Pixel; SCREEN_HEIGHT * SCREEN_WIDTH];

        // Calculate the range of the visible menu
        let mut start_index = 0;
        let screen_max_options = (SCREEN_HEIGHT / GLYPH_HEIGHT) - 1; // -1 for the header
        let mut end_index = std::cmp::min(menu.len(), screen_max_options);
        if selection >= end_index{
            end_index = selection + 1;
            start_index = end_index - screen_max_options;
        }

        let mut frame_buffer_height_index = 0;
        Self::render_string(header, &mut frame_buffer, frame_buffer_height_index, HEADER_COLOR, BACKGROUND_COLOR);
        frame_buffer_height_index += GLYPH_HEIGHT;
        for option_index in start_index..end_index{
            let prompt = &menu[option_index].prompt;
            let (color, bg) = if selection == option_index{(BACKGROUND_COLOR, TEXT_COLOR)}else{(TEXT_COLOR, BACKGROUND_COLOR)};
            Self::render_string(prompt, &mut frame_buffer, frame_buffer_height_index, color, bg);
            frame_buffer_height_index += GLYPH_HEIGHT;
        }
        self.device.swap_buffer(&frame_buffer);
    }
}

impl<'a, GFX:GfxDevice> GfxDeviceMenuRenderer<'a, GFX>{
    fn render_string<S:AsRef<str>>(prompt: S, frame_buffer: &mut [Pixel; SCREEN_HEIGHT * SCREEN_WIDTH], frame_buffer_height_index: usize, color:Color, bg:Color) {
        let mut width_index = 0;
        for char in prompt.as_ref().as_bytes(){
            let glyph = FONT_LUT[(char - FONT_ASCII_START_INDEX) as usize];
            for i in 0..GLYPH_HEIGHT{
                for j in 0..GLYPH_WIDTH{
                    let color = if glyph[i * GLYPH_WIDTH + j] {color}else{bg};
                    frame_buffer[(frame_buffer_height_index + i) * SCREEN_WIDTH + width_index + j] = Pixel::from(color);
                }
            }
            width_index += GLYPH_WIDTH;
    
            // if the string it too long cut it 
            if width_index >= SCREEN_WIDTH{
                break;
            }
        }
    }
}