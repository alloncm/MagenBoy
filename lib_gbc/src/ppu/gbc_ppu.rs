use crate::mmu::memory::Memory;
use crate::utils::color::Color;
use crate::utils::colors::*;
use crate::utils::vec2::Vec2;
use crate::utils::bit_masks::BIT_4_MASK;
use std::cmp;

pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_WIDTH: usize = 160;
const FRAME_BUFFER_SIZE: usize = 0x10000;
//const SPRITE_NORMAL_SIZE:u8 = 8;
const SPRITES_SIZE: usize = 256;
const DRAWING_CYCLE_CLOCKS: u8 = 114;
const LY_MAX_VALUE:u8 = 154;

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
    pub screen_buffer: [u32; SCREEN_HEIGHT*SCREEN_WIDTH],
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
    pub window_scroll: Vec2<u8>,
    pub bg_color_mapping: [Color; 4],
    pub obj_color_mapping0: [Option<Color>;4],
    pub obj_color_mapping1: [Option<Color>;4],
    pub current_line_drawn: Option<u8>
}

impl Default for GbcPpu {
    fn default() -> GbcPpu {
        GbcPpu {
            background_enabled: false,
            background_scroll: Vec2::<u8> { x: 0, y: 0 },
            window_scroll: Vec2::<u8> { x: 0, y: 0 },
            background_tile_map_address: false,
            gbc_mode: false,
            screen_buffer: [0; SCREEN_HEIGHT*SCREEN_WIDTH],
            screen_enable: false,
            sprite_enable: false,
            sprite_extended: false,
            window_enable: false,
            window_tile_background_map_data_address: false,
            window_tile_map_address: false,
            bg_color_mapping: [WHITE, LIGHT_GRAY, DARK_GRAY, BLACK],
            obj_color_mapping0: [None, Some(LIGHT_GRAY), Some(DARK_GRAY), Some(BLACK)],
            obj_color_mapping1: [None, Some(LIGHT_GRAY), Some(DARK_GRAY), Some(BLACK)],
            current_line_drawn:None
        }
    }
}

impl GbcPpu {
    fn color_as_uint(color: &Color) -> u32 {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }

    pub fn get_frame_buffer(&self)->&[u32;SCREEN_HEIGHT*SCREEN_WIDTH]{
        return &self.screen_buffer;
    }

    fn update_ly(&mut self, cycle_counter:u32){
        
        let line = cycle_counter/DRAWING_CYCLE_CLOCKS as u32;
        if line>LY_MAX_VALUE as u32{
            self.current_line_drawn = Some(LY_MAX_VALUE);
        }
        else{
            self.current_line_drawn = Some(line as u8);
        }
    }

    pub fn update_gb_screen(&mut self, memory: &dyn Memory, cycle_counter:u32){
        let last_ly = self.current_line_drawn;
        self.update_ly(cycle_counter);
        if last_ly==self.current_line_drawn{
            return;
        }
        else if (self.current_line_drawn.unwrap() as usize) < SCREEN_HEIGHT{
            let temp = self.current_line_drawn.unwrap();
            let sprites = self.get_bg_and_window_sprites(memory);
            //let obj_sprites = self.get_objects_sprites(memory);
            let bg_frame_buffer = self.get_bg_frame_buffer(&sprites, memory);
            //let window_frame_buffer = self.get_window_frame_buffer(&sprites, memory);
            //let obj_buffer = self.get_objects_frame_buffer(memory, &obj_sprites);
            /*
            for i in 0..window_frame_buffer.len() {
                match &window_frame_buffer[i] {
                    Some(color) => buffer[i] = Self::color_as_uint(color),
                    _ => {}
                }
            }
            for i in 0..obj_buffer.len() {
                match &obj_buffer[i] {
                    Some(color) => buffer[i] = Self::color_as_uint(color),
                    _ => {}
                }
            }
            */
            let line_index = self.current_line_drawn.unwrap() as usize * SCREEN_WIDTH;

            for i in line_index..line_index+SCREEN_WIDTH{
                self.screen_buffer[i] = Self::color_as_uint(&bg_frame_buffer[(i - line_index)]);
            }
        }
    }

    fn get_bg_and_window_sprites(&self, memory: &dyn Memory) -> Vec<Sprite> {
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
                    let mask = 1 << k;
                    let mut value = (byte & (mask)) >> k;
                    value |= (next & (mask) >> k) << 1;
                    if value !=0{
                        //println!("found");
                    }
                    let swaped = 7 - k;
                    sprites[(sprite_number) as usize].pixels[(byte_number * 8 + swaped) as usize] =
                        value;
                }

                byte_number += 1;
            }

            sprite_number += 1;
        }

        return sprites;
    }

    fn get_bg_frame_buffer(&self, sprites: &Vec<Sprite>, memory: &dyn Memory)-> [Color;SCREEN_WIDTH] {
        let current_line = self.current_line_drawn.unwrap();

        let address = if self.background_tile_map_address {
            0x9C00
        } else {
            0x9800
        };
        let mut line_sprites:Vec<Sprite> = Vec::with_capacity(32);
        let index = (current_line as u16 + self.background_scroll.y as u16) / 8;
        if self.window_tile_background_map_data_address {
            for i in index..index + 32 {
                let chr: u8 = memory.read(address + (index*32) + i);
                if chr != 0{
                    //println!("char != 0");
                }
                let sprite = sprites[chr as usize].clone();
                    line_sprites.push(sprite);
            }
        } 
        else {
            for i in index..index + 32 {
                let mut chr: u8 = memory.read(address + (index*32) + i);
                chr = chr.wrapping_add(0x80);
                if chr != 0x80 && chr != 160{
                    println!("char != 80");
                }
                let sprite = sprites[chr as usize].clone();
                line_sprites.push(sprite);
            }
        }   

        let mut drawn_line:[Color; 256] = [Color::default();256];

        let sprite_line = current_line % 8;
        for i in 0..line_sprites.len(){
            for j in 0..8{
                let pixel = line_sprites[i].pixels[(sprite_line * 8 + j) as usize];
                drawn_line[(i * 8) + j as usize] = self.get_bg_color(pixel);
            }
        }

        let mut screen_line:[Color;SCREEN_WIDTH] = [Color::default();SCREEN_WIDTH];
        let end = cmp::min(self.background_scroll.x as usize + SCREEN_WIDTH,256);
        for i in self.background_scroll.x as usize..end{
            screen_line[(i - self.background_scroll.x as usize) as usize] = drawn_line[i as usize];
        }
        
        return screen_line;
    }

    fn get_window_frame_buffer(&self, sprites: &Vec<Sprite>, memory: &dyn Memory,)-> Vec<Option<Color>> {
        let mut frame_buffer: Vec<Sprite> = Vec::with_capacity(sprites.len());
        for _ in 0..frame_buffer.capacity() {
            frame_buffer.push(Sprite::new());
        }

        let address = if self.window_tile_map_address {
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

        for i in 0..32 {
            for k in 0..8 {
                for j in 0..32 {
                    for n in 0..8 {
                        let colors_buffer_address = (i * 32 * 64) + k * 256 + j * 8 + n;
                        let frame_buffer_address = i * 32 + j;
                        let pixel_index = k * 8 + n;
                        let color_index = frame_buffer[frame_buffer_address].pixels[pixel_index];
                        let color = self.get_bg_color(color_index);
                        colors_buffer[colors_buffer_address] = color;
                    }
                }
            }
        }

        let mut other_frame_buffer: Vec<Option<Color>> =
            Vec::with_capacity(SCREEN_HEIGHT * SCREEN_WIDTH);
        for _ in 0..other_frame_buffer.capacity() {
            other_frame_buffer.push(Option::None);
        }

        let start_y: usize = cmp::max(self.window_scroll.y, self.background_scroll.y) as usize;
        let end_y: usize = cmp::min(self.window_scroll.y, self.background_scroll.y) as usize + SCREEN_HEIGHT;
        let start_x: usize = cmp::max(self.window_scroll.x, self.background_scroll.x) as usize;
        let end_x: usize = cmp::min(self.window_scroll.x, self.background_scroll.x) as usize + SCREEN_WIDTH;
        for i in start_y..end_y {
            for j in start_x..end_x {
                let index = ((i - start_y) * SCREEN_WIDTH) + (j - start_x);
                other_frame_buffer[index] = Option::Some(colors_buffer[((i as u16) * 256 + j as u16) as usize].clone());
            }
        }

        return other_frame_buffer;
    }

    fn get_objects_sprites(&self, memory: &dyn Memory) -> Vec<Sprite> {
        let mut sprites: Vec<Sprite> = Vec::with_capacity(SPRITES_SIZE);
        for _ in 0..sprites.capacity() {
            sprites.push(Sprite::new());
        }
        let address = 0x8000;

        let mut sprite_number = 0;
        for i in (0..0x1000).step_by(16) {
            let mut byte_number = 0;
            for j in (i..i + 16).step_by(2) {
                let byte = memory.read(address + j);
                let next = memory.read(address + j + 1);
                for k in 0..8 {
                    let mask = 1 << k;
                    let mut value = (byte & (mask)) >> k;
                    value |= (next & (mask) >> k) << 1;
                    let swaped = 7 - k;
                    sprites[(sprite_number) as usize].pixels[(byte_number * 8 + swaped) as usize] =
                        value;
                }

                byte_number += 1;
            }

            sprite_number += 1;
        }

        return sprites;
    }

    fn get_objects_frame_buffer(&self, memory:&dyn Memory, sprites:&Vec<Sprite>)->Vec<Option<Color>>{
        let oam_address = 0xFE00;

        let mut frame_buffer: Vec<Option<Color>> = Vec::with_capacity(FRAME_BUFFER_SIZE);
        for _ in 0..frame_buffer.capacity() {
            frame_buffer.push(Option::None);
        }

        for i in (0..0x100).step_by(4){
            let end_y = memory.read(oam_address + i);
            let end_x = memory.read(oam_address + i + 1);
            let tile_number = memory.read(oam_address + i + 2);
            let attributes = memory.read(oam_address + i + 3);
            let sprite = &sprites[tile_number as usize];
            let start_y = cmp::max(0, (end_y as i16) - 16) as u8;
            let start_x = cmp::max(0, (end_x as i16) - 8) as u8;
            for y in start_y..end_y{
                for x in start_x..end_x{
                    let color = self.get_obj_color(sprite.pixels[((y-start_y)*8+(x-start_x)) as usize],(attributes & BIT_4_MASK) != 0);
                    frame_buffer[(y as u16 *256 + x as u16) as usize] = color;
                }
            }
        
        }

        let mut screen_buffer: Vec<Option<Color>> = Vec::with_capacity(SCREEN_HEIGHT*SCREEN_WIDTH);

        
        let end_y = cmp::min(self.background_scroll.y as u16 + SCREEN_HEIGHT as u16, 255) as u8;
        let end_x = cmp::min(self.background_scroll.x as u16 + SCREEN_WIDTH as u16, 255) as u8;
        for i in self.background_scroll.y..=end_y {
            for j in self.background_scroll.x..=end_x {
                screen_buffer.push(frame_buffer[((i as u16) * 256 + j as u16) as usize].clone());
            }
        }

        return screen_buffer;
    }

    fn get_bg_color(&self, color: u8) -> Color {
        return self.bg_color_mapping[color as usize].clone();
    }

    fn get_obj_color(&self, color:u8, pallet_bit_set:bool)->Option<Color>{
        return if pallet_bit_set{
            self.obj_color_mapping1[color as usize].clone()
        }
        else{
            self.obj_color_mapping0[color as usize].clone()
        };
    }
}
