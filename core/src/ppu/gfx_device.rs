use super::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

/// Pixel is in the format of RGB565 even though the CGB stores pixels as RGB555 as the gbdev docs indicates as RGB565 is much more used format now days
/// The bits are represented as: RGB565 (low bits (Red) -> high bits (Blue))
pub type Pixel = u16;

pub trait GfxDevice{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]);
}