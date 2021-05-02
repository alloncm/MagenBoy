use crate::utils::bit_masks::*;

pub struct SpriteAttribute {
    pub y: u8,
    pub x: u8,
    pub tile_number: u8,
    pub is_bg_priority: bool,
    pub flip_y: bool,
    pub flip_x: bool,
    pub palette_number: bool,
}

impl SpriteAttribute {
    pub fn new(y: u8, x: u8, tile_number: u8, attributes: u8) -> Self {
        SpriteAttribute {
            y: y,
            x: x,
            tile_number: tile_number,
            is_bg_priority: attributes & BIT_7_MASK != 0,
            flip_y: attributes & BIT_6_MASK != 0,
            flip_x: attributes & BIT_5_MASK != 0,
            palette_number: attributes & BIT_4_MASK != 0,
        }
    }
}
