use super::sprite::Sprite;

pub struct NormalSprite {
    pub pixels: [u8; 64],
}

impl NormalSprite {
    const SIZE: u8 = 8;

    pub fn new() -> NormalSprite {
        NormalSprite { pixels: [0; 64] }
    }

    fn copy_pixels(sprite: &mut NormalSprite, index: usize, pixels: &[u8]) {
        for i in 0..pixels.len() {
            sprite.pixels[index * 8 + i] = pixels[i];
        }
    }
}

impl Clone for NormalSprite {
    fn clone(&self) -> Self {
        NormalSprite {
            pixels: self.pixels,
        }
    }
}

impl Sprite for NormalSprite {
    fn size(&self) -> u8 {
        Self::SIZE
    }

    fn get_pixel(&self, pos: u8) -> u8 {
        self.pixels[pos as usize]
    }

    fn set_pixel(&mut self, pos: u8, pixel: u8) {
        self.pixels[pos as usize] = pixel;
    }

    fn flip_x(&mut self) {
        let mut fliiped = NormalSprite::new();

        for y in 0..8 {
            let line = &self.pixels[y * 8..(y + 1) * 8];
            for x in 0..4 {
                fliiped.pixels[y * 8 + x] = line[7 - x];
                fliiped.pixels[y * 8 + (7 - x)] = line[x];
            }
        }

        *self = fliiped;
    }

    fn flip_y(&mut self) {
        let mut flipped = NormalSprite::new();
        for y in 0..4 {
            let upper_line = &self.pixels[y * 8..(y + 1) * 8];
            let opposite_index = 7 - y;
            let lower_line = &self.pixels[opposite_index * 8..(opposite_index + 1) * 8];

            Self::copy_pixels(&mut flipped, y, lower_line);
            Self::copy_pixels(&mut flipped, opposite_index, upper_line);
        }

        *self = flipped;
    }
}
