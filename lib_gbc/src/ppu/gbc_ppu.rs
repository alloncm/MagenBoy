use crate::mmu::memory::Memory;
use crate::utils::color::Color;
use crate::utils::vec2::Vec2;
use crate::utils::colors::*;

const SCREEN_HEIGHT: usize = 144;
const SCREEN_WIDTH: usize = 160;
const FRAME_BUFFER_SIZE: usize = 0x10000;
//const SPRITE_NORMAL_SIZE:u8 = 8;
const SPRITES_SIZE: usize = 32 * 32;

#[derive(Clone)]
struct Sprite {
    pixels: [u8; 64],
}

impl Sprite {
    pub fn new() -> Sprite {
        Sprite { pixels: [0; 64] }
    }
}

pub struct GbcPpu {
    pub screen_buffer: [u8; FRAME_BUFFER_SIZE],
    pub screen_enable: bool,
    pub window_enable: bool,
    pub sprite_extended: bool,
    pub background_enabled: bool,
    pub gbc_mode: bool,
    pub sprite_enable: bool,
    pub window_tile_map_address: bool,
    pub window_tile_background_map_data_address: bool,
    pub background_tile_map_address: bool,
    pub background_scroll: Vec2<u8>,
    pub colors_mapping:[Color;4]
}

impl Default for GbcPpu {
    fn default() -> GbcPpu {
        GbcPpu {
            background_enabled: false,
            background_scroll: Vec2::<u8> { x: 0, y: 0 },
            background_tile_map_address: false,
            gbc_mode: false,
            screen_buffer: [0; FRAME_BUFFER_SIZE],
            screen_enable: false,
            sprite_enable: false,
            sprite_extended: false,
            window_enable: false,
            window_tile_background_map_data_address: false,
            window_tile_map_address: false,
            colors_mapping:[WHITE, LIGHT_GRAY, DARK_GRAY, BLACK]
        }
    }
}

impl GbcPpu {
    

    pub fn get_gb_screen(&self, memory: &dyn Memory) -> Vec<u32> {
        let sprites = self.get_bg_sprites(memory);
        let frame_buffer = self.get_bg_frame_buffer(sprites, memory);
        let mut buffer = Vec::<u32>::new();
        for color in frame_buffer.iter() {
            let value: u32 = ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32);
            buffer.push(value);
        }

        return buffer;
    }

    fn get_bg_sprites(&self, memory: &dyn Memory) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = Vec::with_capacity(SPRITES_SIZE);
        for _ in 0..sprites.capacity() {
            sprites.push(Sprite::new());
        }
        let address = if self.window_tile_background_map_data_address {
            0x8000
        } else {
            0x8800
        };

        let mut sprite_number = 0;
        for i in (0..0x1000).step_by(16) {
            let mut byte_number = 0;
            for j in (i..i + 16).step_by(2) {
                let byte = memory.read(address + j);
                let next = memory.read(address + j + 1);
                for k in 0..8 {
                    let mask = 1<<k;
                    let mut value = (byte & (mask)) >> k;
                    value |= (next & (mask) >> k) << 1;
                    let swaped = 7-k;
                    sprites[(sprite_number) as usize].pixels[(byte_number * 8 + swaped) as usize] = value;
                }

                byte_number += 1;
            }

            sprite_number+=1;
        }

        return sprites;
    }

    fn get_bg_frame_buffer(&self, sprites: Vec<Sprite>, memory: &dyn Memory) -> Vec<Color> {
        let mut frame_buffer: Vec<Sprite> = Vec::with_capacity(sprites.len());
        for _ in 0..frame_buffer.capacity() {
            frame_buffer.push(Sprite::new());
        }

        let address = if self.background_tile_map_address {
            0x9C00
        } else {
            0x9800
        };

        if self.window_tile_background_map_data_address {
            for i in 0..0x400 {
                let chr: u8 = memory.read(address + i);
                let sprite = sprites[chr as usize].clone();
                frame_buffer[i as usize] = sprite;
            }
        } else {
            for i in 0..0x400 {
                let mut chr: u8 = memory.read(address + i);
                chr = chr.wrapping_add(0x80);
                let sprite = sprites[chr as usize].clone();
                frame_buffer[i as usize] = sprite;
            }
        }

        let mut colors_buffer: Vec<Color> = Vec::with_capacity(FRAME_BUFFER_SIZE);
        for _ in 0..colors_buffer.capacity() {
            colors_buffer.push(Color::default());
        }

        for i in 0..32{
            for k in 0..8{
                for j in 0..32{
                    for n in 0..8{
                        let colors_buffer_address = (i*32*64) + k*256 + j*8 + n;
                        let frame_buffer_address = i*32+j;
                        let pixel_index = k*8+n;
                        let color_index = frame_buffer[frame_buffer_address].pixels[pixel_index];
                        let color = self.get_color(color_index);
                        colors_buffer[colors_buffer_address] = color;
                    }
                }
            }
        }

        let mut other_frame_buffer: Vec<Color> = Vec::new();
        for _ in 0..other_frame_buffer.capacity() {
            other_frame_buffer.push(Color::default());
        }

        for i in self.background_scroll.y..self.background_scroll.y + SCREEN_HEIGHT as u8 {
            for j in self.background_scroll.x..self.background_scroll.x + SCREEN_WIDTH as u8 {
                other_frame_buffer
                    .push(colors_buffer[((i as u16) * 256 + j as u16) as usize].clone());
            }
        }

        return other_frame_buffer;
    }

    fn get_color(&self, color: u8) -> Color {
        return self.colors_mapping[color as usize].clone();
    }
}
