use super::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

/// Pixel is in the format of RGB555 as in the gbdev docs which is low bits (Red) -> high bits (Blue)
pub type Pixel = u16;

pub trait GfxDevice{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]);
}