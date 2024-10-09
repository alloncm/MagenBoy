use super::gfx_device::Pixel;

pub const WHITE:Color = Color {r: 255,g: 255,b: 255};
pub const LIGHT_GRAY:Color = Color {r: 160,g: 160,b: 160};
pub const DARK_GRAY:Color = Color {r: 64,g: 64,b: 64};
pub const BLACK:Color = Color {r: 0,g: 0,b: 0};

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Color{
    pub r:u8,
    pub g:u8,
    pub b:u8
}

impl From<Color> for Pixel{
    #[inline]
    fn from(color: Color) -> Self {(((color.b >> 3) as u16) << 10) | (((color.g >> 3) as u16) << 5) | ((color.r >> 3) as u16)}
}

impl From<u16> for Color{
    // color is RGB555 u16 value
    fn from(color:u16)->Color{
        Color{
            r: ((color & 0b1_1111) as u8) << 3, 
            g: (((color >> 5) & 0b1_1111) as u8) << 3, 
            b: (((color >> 10) & 0b1_1111) as u8) << 3
        }
    }
}