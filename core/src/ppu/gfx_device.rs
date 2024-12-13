use super::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

/// Pixel is in the format of RGB565 even though the CGB stores pixels as BGR555 as the gbdev docs indicates, RGB565 is much more used format now days
/// The bits are represented as: RGB565 (low bits (BLue) -> high bits (Red))
pub type Pixel = u16;

pub trait GfxDevice{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]);
}