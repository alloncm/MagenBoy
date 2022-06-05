use super::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[cfg(not(feature = "compact-pixel"))]
pub type Pixel = u32;
#[cfg(feature = "compact-pixel")]
pub type Pixel = u16;


pub trait GfxDevice{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]);
}