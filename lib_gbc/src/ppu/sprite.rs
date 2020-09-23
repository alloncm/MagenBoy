pub struct Sprite {
    pub pixels: [u8; 64],
}

impl Sprite {
    pub fn new() -> Sprite {
        Sprite { pixels: [0; 64] }
    }
}

impl Clone for Sprite{
    fn clone(&self)->Self{
        Sprite{
            pixels:self.pixels
        }
    }
}