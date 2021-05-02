use super::sprite::*;

pub struct ExtendedSprite {
    pub pixels: [u8; 128],
}

impl ExtendedSprite {
    const SIZE: u8 = 16;

    pub fn new() -> ExtendedSprite {
        ExtendedSprite { pixels: [0; 128] }
    }
}

impl Clone for ExtendedSprite {
    fn clone(&self) -> Self {
        ExtendedSprite {
            pixels: self.pixels,
        }
    }
}

impl Sprite for ExtendedSprite {
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
        let mut fliiped = ExtendedSprite::new();

        for y in 0..16 {
            let line = &self.pixels[y * 8..(y + 1) * 8];
            for x in 0..4 {
                fliiped.pixels[y * 8 + x] = line[7 - x];
                fliiped.pixels[y * 8 + (7 - x)] = line[x];
            }
        }

        *self = fliiped;
    }

    fn flip_y(&mut self) {
        let mut flipped = ExtendedSprite::new();
        for y in 0..8 {
            let upper_line = &self.pixels[y * 8..(y + 1) * 8];
            let opposite_index = 15 - y;
            let lower_line = &self.pixels[opposite_index * 8..(opposite_index + 1) * 8];

            copy_pixels(&mut flipped, y as u8, lower_line);
            copy_pixels(&mut flipped, opposite_index as u8, upper_line);
        }

        *self = flipped;
    }
}
