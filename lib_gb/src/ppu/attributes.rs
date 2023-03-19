use crate::utils::bit_masks::*;

#[derive(Clone, Copy, Default)]
pub struct Attributes{
    pub bg_priority:bool,
    pub flip_y:bool,
    pub flip_x:bool,
    pub gbc_bank:bool,
}

impl Attributes{
    pub const fn new(attribute:u8)->Self{
        Self{
            bg_priority: (attribute & BIT_7_MASK) != 0,
            flip_y: (attribute & BIT_6_MASK) != 0,
            flip_x: (attribute & BIT_5_MASK) != 0,
            gbc_bank:(attribute & BIT_3_MASK) != 0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct GbcBackgroundAttributes{
    pub attribute:Attributes,
    pub cgb_pallete_number:u8,
}

impl GbcBackgroundAttributes{
    pub const fn new(attribute:u8)->Self{
        Self{
            attribute: Attributes::new(attribute),
            cgb_pallete_number: attribute & 0b111
        }
    }
}

pub struct SpriteAttributes{
    pub y:u8,
    pub x:u8,
    pub tile_number:u8,
    pub gb_palette_number:bool,
    pub gbc_palette_number:u8,
    pub attributes:Attributes,
    pub visibility_start:u8,
    pub visibility_end:u8,
}

impl SpriteAttributes{
    pub fn new(y:u8, x:u8, tile_number:u8, attributes:u8, visibility_start:u8, visibility_end:u8)->Self{
        Self{
            y, x, tile_number, attributes: Attributes::new(attributes), 
            gb_palette_number: (attributes & BIT_4_MASK) != 0, gbc_palette_number: attributes & 0b111,
            visibility_end, visibility_start
        }
    }
}