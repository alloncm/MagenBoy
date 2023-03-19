use super::gfx_device::Pixel;

pub const WHITE:Color = Color {r: 255,g: 255,b: 255};
pub const LIGHT_GRAY:Color = Color {r: 160,g: 160,b: 160};
pub const DARK_GRAY:Color = Color {r: 64,g: 64,b: 64};
pub const BLACK:Color = Color {r: 0,g: 0,b: 0};

pub struct Color{
    pub r:u8,
    pub g:u8,
    pub b:u8
}

impl Default for Color{
    fn default()->Color{
        Color{
            r:0,
            g:0,
            b:0
        }
    }
}

impl Clone for Color{
    fn clone(&self)->Color{
        Color{
            r:self.r,
            g:self.g,
            b:self.b
        }
    }
}

impl Copy for Color{}

impl PartialEq for Color{
    fn eq(&self,color:&Color)->bool{
        self.b == color.b && 
        self.g == color.g && 
        self.r == color.r
    }
}

impl From<Color> for Pixel{
    #[inline]
    fn from(color: Color) -> Self {
        #[cfg(not(feature = "u16pixel"))]
        {
            ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
        }
        #[cfg(feature = "u16pixel")]
        {
            (((color.r >> 3) as u16) << 11) | (((color.g >> 2) as u16) << 5) | ((color.b >> 3) as u16)
        }
    }
}

impl From<u16> for Color{
    // color is RGB555 u16 value
    fn from(color:u16)->Color{
        Color{
            r:(color as u8 & 0b1_1111)<<3, 
            g: ((color >> 5) as u8 & 0b1_1111)<<3, 
            b: ((color >> 10) as u8 & 0b1_1111)<<3
        }
    }
}