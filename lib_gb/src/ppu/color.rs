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

impl From<Color> for u32{
    fn from(color: Color) -> Self {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }
}