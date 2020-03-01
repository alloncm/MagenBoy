use crate::utils::vec2::Vec2;

const SCREEN_HEIGHT: usize = 144;
const SCREEN_WIDTH: usize = 160;
const SCREEN_BUFFER_SIZE: usize = 0xFF*0xFF;

pub struct GbcPpu{
    pub screen_cordinates: Vec2<u8>,
    pub window_cordinates: Vec2<u8>,
    pub screen_buffer:[u8;SCREEN_BUFFER_SIZE],
    pub screen_enable:bool,
    pub windows_enable:bool,
    pub sprite_extended:bool,
    pub background_enabled:bool
}

impl GbcPpu{
    pub fn get_screen_buffer(&self)->[u8;SCREEN_HEIGHT*SCREEN_WIDTH]{
        return [0;23040];
    }
}