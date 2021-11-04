use super::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub trait GfxDevice{
    fn swap_buffer(&mut self, buffer:&[u32; SCREEN_HEIGHT * SCREEN_WIDTH]);
}