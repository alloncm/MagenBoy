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
    // Notice that Pixel is RGB565, for more see Pixel doc
    fn from(color: Color) -> Self {(((color.r >> 3) as u16) << 11) | (((color.g >> 2) as u16) << 5) | ((color.b >> 3) as u16)}
}

impl From<u16> for Color{
    /// color is BGR555 u16 value (Red - low bits, Blue - High bits)
    fn from(color:u16)->Color{
        Color{
            r: ((color & 0b1_1111) as u8) << 3, 
            g: (((color >> 5) & 0b1_1111) as u8) << 3, 
            b: (((color >> 10) & 0b1_1111) as u8) << 3
        }
    }
}

impl From<u32> for Color{
    /// Color is RGB888 value (Red - high bits, Blue - low bits)
    fn from(color: u32) -> Self {
        Self{ r: ((color >> 16) & 0xFF) as u8, g: ((color >> 8) & 0xFF) as u8, b: (color & 0xFF) as u8 }
    }
}