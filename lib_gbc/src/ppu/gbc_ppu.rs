use crate::mmu::memory::Memory;
use super::ppu_state::PpuState;
use crate::utils::color::Color;
use crate::utils::colors::*;
use crate::utils::vec2::Vec2;
use crate::utils::bit_masks::BIT_4_MASK;
use crate::utils::colors::WHITE;
use super::sprite::Sprite;
use std::cmp;

pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_WIDTH: usize = 160;
const FRAME_BUFFER_SIZE: usize = 0x10000;
//const SPRITE_NORMAL_SIZE:u8 = 8;
const SPRITES_SIZE: usize = 256;

const OAM_CLOCKS:u8 = 20;
const PIXEL_TRANSFER_CLOCKS:u8 = 43;
const H_BLANK_CLOCKS:u8 = 51;
const DRAWING_CYCLE_CLOCKS: u8 = OAM_CLOCKS + H_BLANK_CLOCKS + PIXEL_TRANSFER_CLOCKS;
const LY_MAX_VALUE:u8 = 154;



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
    pub current_line_drawn: u8,
    pub state:PpuState,

    current_cycle:u32,
    line_rendered:bool
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
            current_line_drawn:0,
            state:PpuState::OamSearch,
            line_rendered:false,
            current_cycle:0
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

    fn update_ly(&mut self){
        
        let line = self.current_cycle/DRAWING_CYCLE_CLOCKS as u32;
        if line>LY_MAX_VALUE as u32{
            self.current_line_drawn = LY_MAX_VALUE;
            self.line_rendered = true;
            self.current_cycle = 0;
        }
        else if self.current_line_drawn != line as u8{
            self.current_line_drawn = line as u8;
            self.line_rendered = false;
        }
    }

    fn get_ppu_state(cycle_counter:u32, last_ly:u8)->PpuState{
        if last_ly > SCREEN_HEIGHT as u8{
            return PpuState::Vblank;
        }

        //getting the reminder of the clocks 
        let current_line_clocks = cycle_counter % DRAWING_CYCLE_CLOCKS as u32;
        
        const PIXEL_TRANSFER_START:u8 = OAM_CLOCKS+1;
        const PIXEL_TRANSFER_END:u8 = OAM_CLOCKS + PIXEL_TRANSFER_CLOCKS;
        const H_BLANK_START:u8 = PIXEL_TRANSFER_END+1;
        const H_BLANK_END:u8 = PIXEL_TRANSFER_END + H_BLANK_CLOCKS;

        return match current_line_clocks as u8{
            0 ..= OAM_CLOCKS => PpuState::OamSearch,
            PIXEL_TRANSFER_START ..= PIXEL_TRANSFER_END => PpuState::PixelTransfer,
            H_BLANK_START ..= H_BLANK_END => PpuState::Hblank,
            _=>std::panic!("Error calculating ppu state")
        };
    }

    pub fn update_gb_screen(&mut self, memory: &dyn Memory, cycles_passed:u8){
        if !self.screen_enable{
            self.current_cycle = 0;
            self.current_line_drawn = 0;
            self.screen_buffer = [Self::color_as_uint(&WHITE);SCREEN_HEIGHT * SCREEN_WIDTH];
            self.state = PpuState::Hblank;
            return;
        }

        self.current_cycle += cycles_passed as u32;
        self.update_ly();
        self.state = Self::get_ppu_state(self.current_cycle, self.current_line_drawn);

        if self.state as u8 != PpuState::PixelTransfer as u8{
            return;
        }

        if !self.line_rendered &&  (self.current_line_drawn as usize) < SCREEN_HEIGHT{
            self.line_rendered = true;
            let temp = self.current_line_drawn;
            //let obj_sprites = self.get_objects_sprites(memory);
            let bg_frame_buffer_line = self.get_bg_frame_buffer(memory);
            let window_frame_buffer_line = self.get_window_frame_buffer(memory);
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
            let line_index = self.current_line_drawn as usize * SCREEN_WIDTH;
            for i in line_index..line_index+SCREEN_WIDTH{
                self.screen_buffer[i] = Self::color_as_uint(&bg_frame_buffer_line[(i - line_index)]);

                match window_frame_buffer_line[(i - line_index)]{
                    Some(val)=>self.screen_buffer[i] = Self::color_as_uint(&val),
                    None=>{}
                }
            }
        }

    }

    fn get_bg_frame_buffer(&self, memory: &dyn Memory)-> [Color;SCREEN_WIDTH] {
        if !self.background_enabled{
            //color in BGP 0
            let color = self.get_bg_color(0);
            return [color;SCREEN_WIDTH]
        }

        let current_line = self.current_line_drawn;

        let address = if self.background_tile_map_address {
            0x9C00
        } else {
            0x9800
        };
        let mut line_sprites:Vec<Sprite> = Vec::with_capacity(32);
        let index = ((current_line.wrapping_add(self.background_scroll.y)) / 8) as u16;
        if self.window_tile_background_map_data_address {
            for i in 0..32 {
                let chr: u8 = memory.read(address + (index*32) + i);
                let sprite = Self::get_bg_sprite(chr, memory, 0x8000);
                line_sprites.push(sprite);
            }
        } 
        else {
            for i in 0..32 {
                let mut chr: u8 = memory.read(address + (index*32) + i);
                chr = chr.wrapping_add(0x80);
                let sprite = Self::get_bg_sprite(chr, memory, 0x8800);
                line_sprites.push(sprite);
            }
        }   

        let mut drawn_line:[Color; 256] = [Color::default();256];

        let sprite_line = (current_line as u16 + self.background_scroll.y as u16) % 8;
        for i in 0..line_sprites.len(){
            for j in 0..8{
                let pixel = line_sprites[i].pixels[((sprite_line * 8) + j) as usize];
                drawn_line[(i * 8) + j as usize] = self.get_bg_color(pixel);
            }
        }

        let mut screen_line:[Color;SCREEN_WIDTH] = [Color::default();SCREEN_WIDTH];
        for i in 0..SCREEN_WIDTH{
            let index:usize = (i as u8).wrapping_add(self.background_scroll.x) as usize;
            screen_line[i] = drawn_line[index]
        }
        
        return screen_line;
    }

    fn get_bg_sprite(index:u8, memory:&dyn Memory, data_address:u16)->Sprite{
        let mut sprite = Sprite::new();

        let mut byte_number = 0;
        let start:u16 = index as u16 * 16;
        let end:u16 = start + 16;
        for j in (start .. end).step_by(2) {
            let byte = memory.read(data_address + j);
            let next = memory.read(data_address + j + 1);
            for k in 0..8 {
                let mask = 1 << k;
                let mut value = (byte & (mask)) >> k;
                value |= (next & (mask) >> k) << 1;
                let swaped = 7 - k;
                sprite.pixels[(byte_number * 8 + swaped) as usize] = value;
            }

            byte_number += 1;
        }

        return sprite;
    }

    
    fn get_window_frame_buffer(&self, memory: &dyn Memory,)-> [Option<Color>; SCREEN_WIDTH] {
        if !self.window_enable || self.current_line_drawn < self.window_scroll.y{
            return [Option::None; SCREEN_WIDTH];
        }

        let current_line = self.current_line_drawn;

        let address = if self.window_tile_map_address {
            0x9C00
        } else {
            0x9800
        };
        let mut line_sprites:Vec<Sprite> = Vec::with_capacity(32);
        let index = ((current_line.wrapping_add(self.background_scroll.y)) / 8) as u16;
        if self.window_tile_background_map_data_address {
            for i in 0..32 {
                let chr: u8 = memory.read(address + (index*32) + i);
                let sprite = Self::get_bg_sprite(chr, memory, 0x8000);
                line_sprites.push(sprite);
            }
        } 
        else {
            for i in 0..32 {
                let mut chr: u8 = memory.read(address + (index*32) + i);
                chr = chr.wrapping_add(0x80);
                let sprite = Self::get_bg_sprite(chr, memory, 0x8800);
                line_sprites.push(sprite);
            }
        }   

        let mut drawn_line:[Color; 256] = [Color::default();256];

        let sprite_line = (current_line as u16 + self.window_scroll.y as u16) % 8;
        for i in 0..line_sprites.len(){
            for j in 0..8{
                let pixel = line_sprites[i].pixels[((sprite_line * 8) + j) as usize];
                drawn_line[(i * 8) + j as usize] = self.get_bg_color(pixel);
            }
        }

        let mut screen_line:[Option<Color>;SCREEN_WIDTH] = [Option::None;SCREEN_WIDTH];
        for i in self.window_scroll.x as usize..SCREEN_WIDTH{
            screen_line[(i as usize)] = Option::Some(drawn_line[i]);
        }
        
        return screen_line;
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
