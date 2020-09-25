use crate::mmu::video_memory::ReadOnlyVideoMemory;
use super::ppu_state::PpuState;
use crate::utils::color::Color;
use crate::utils::colors::*;
use crate::utils::vec2::Vec2;
use crate::utils::colors::WHITE;
use super::normal_sprite::NormalSprite;
use super::extended_sprite::ExtendedSprite;
use super::sprite::Sprite;
use super::sprite_attribute::SpriteAttribute;
use std::cmp;

pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_WIDTH: usize = 160;

const OAM_CLOCKS:u8 = 20;
const PIXEL_TRANSFER_CLOCKS:u8 = 43;
const H_BLANK_CLOCKS:u8 = 51;
const DRAWING_CYCLE_CLOCKS: u8 = OAM_CLOCKS + H_BLANK_CLOCKS + PIXEL_TRANSFER_CLOCKS;
const LY_MAX_VALUE:u8 = 154;
const OAM_ADDRESS:u16 = 0xFE00;
const OAM_SIZE:u16 = 0xA0;
const OBJ_PER_LINE:usize = 10;
const SPRITE_WIDTH:u8 = 8;
const NORMAL_SPRITE_HIEGHT:u8 = 8;
const SPRITE_MAX_HEIGHT:u8 = 16;
const BG_SPRITES_PER_LINE:u16 = 32;
const SPRITE_SIZE_IN_MEMORY:u16 = 16;


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

    window_active:bool,
    window_line_counter:u8,
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
            current_cycle:0,
            window_line_counter:0,
            window_active:false
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
            self.window_line_counter = 0;
            self.window_enable = false;
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

    pub fn update_gb_screen(&mut self, memory: &dyn ReadOnlyVideoMemory, cycles_passed:u8){
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

            let mut frame_buffer_line = self.get_bg_frame_buffer(memory);
            self.draw_window_frame_buffer(memory, &mut frame_buffer_line);
            self.draw_objects_frame_buffer(memory, &mut frame_buffer_line);

            let line_index = self.current_line_drawn as usize * SCREEN_WIDTH;
            
            for i in line_index..line_index+SCREEN_WIDTH{
                self.screen_buffer[i] = Self::color_as_uint(&frame_buffer_line[(i - line_index)]);
            }
        }

    }

    fn get_bg_frame_buffer(&self, memory: &dyn ReadOnlyVideoMemory)-> [Color;SCREEN_WIDTH] {
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
        let mut line_sprites:Vec<NormalSprite> = Vec::with_capacity(BG_SPRITES_PER_LINE as usize);
        let index = ((current_line.wrapping_add(self.background_scroll.y)) / NORMAL_SPRITE_HIEGHT) as u16;
        if self.window_tile_background_map_data_address {
            for i in 0..BG_SPRITES_PER_LINE {
                let chr: u8 = memory.read(address + (index*BG_SPRITES_PER_LINE) + i);
                let sprite = Self::get_normal_sprite(chr, memory, 0x8000);
                line_sprites.push(sprite);
            }
        } 
        else {
            for i in 0..BG_SPRITES_PER_LINE {
                let mut chr: u8 = memory.read(address + (index*BG_SPRITES_PER_LINE) + i);
                chr = chr.wrapping_add(0x80);
                let sprite = Self::get_normal_sprite(chr, memory, 0x8800);
                line_sprites.push(sprite);
            }
        }   

        let mut drawn_line:[Color; 256] = [Color::default();256];

        let sprite_line = (current_line as u16 + self.background_scroll.y as u16) % 8;
        for i in 0..line_sprites.len(){
            for j in 0..SPRITE_WIDTH{
                let pixel = line_sprites[i].pixels[((sprite_line as u8 * SPRITE_WIDTH) + j) as usize];
                drawn_line[(i * SPRITE_WIDTH as usize) + j as usize] = self.get_bg_color(pixel);
            }
        }

        let mut screen_line:[Color;SCREEN_WIDTH] = [Color::default();SCREEN_WIDTH];
        for i in 0..SCREEN_WIDTH{
            let index:usize = (i as u8).wrapping_add(self.background_scroll.x) as usize;
            screen_line[i] = drawn_line[index]
        }
        
        return screen_line;
    }

    fn get_normal_sprite(index:u8, memory:&dyn ReadOnlyVideoMemory, data_address:u16)->NormalSprite{
        let mut sprite = NormalSprite::new();

        let mut line_number = 0;
        let start:u16 = index as u16 * SPRITE_SIZE_IN_MEMORY;
        let end:u16 = start + SPRITE_SIZE_IN_MEMORY;
        for j in (start .. end).step_by(2) {
            Self::get_line(memory, &mut sprite, data_address + j, line_number);
            line_number += 1;
        }

        return sprite;
    }

    
    fn draw_window_frame_buffer(&mut self, memory: &dyn ReadOnlyVideoMemory, line:&mut [Color;SCREEN_WIDTH]) {
        if !self.window_enable || !self.background_enabled || self.current_line_drawn < self.window_scroll.y{ 
            return;
        }

        if self.current_line_drawn == self.window_scroll.y{
            self.window_active = true;
        }
        
        if !self.window_active {
            return;
        }

        if self.window_scroll.x as usize > SCREEN_WIDTH {    
            return;
        }

        let address = if self.window_tile_map_address {
            0x9C00
        } else {
            0x9800
        };
        let mut line_sprites:Vec<NormalSprite> = Vec::with_capacity(BG_SPRITES_PER_LINE as usize);
        let index = ((self.window_line_counter) / 8) as u16;
        if self.window_tile_background_map_data_address {
            for i in 0..BG_SPRITES_PER_LINE {
                let chr: u8 = memory.read(address + (index*BG_SPRITES_PER_LINE) + i);
                let sprite = Self::get_normal_sprite(chr, memory, 0x8000);
                line_sprites.push(sprite);
            }
        } 
        else {
            for i in 0..BG_SPRITES_PER_LINE {
                let mut chr: u8 = memory.read(address + (index*BG_SPRITES_PER_LINE) + i);
                chr = chr.wrapping_add(0x80);
                let sprite = Self::get_normal_sprite(chr, memory, 0x8800);
                line_sprites.push(sprite);
            }
        }   

        let mut drawn_line:[Color; 256] = [Color::default();256];

        let sprite_line = ( self.window_line_counter) % NORMAL_SPRITE_HIEGHT;
        for i in 0..line_sprites.len(){
            for j in 0..SPRITE_WIDTH{
                let pixel = line_sprites[i].pixels[((sprite_line * SPRITE_WIDTH) + j) as usize];
                drawn_line[(i * SPRITE_WIDTH as usize) + j as usize] = self.get_bg_color(pixel);
            }
        }

        for i in self.window_scroll.x as usize..SCREEN_WIDTH{
            line[(i as usize)] = drawn_line[i - self.window_scroll.x as usize];
        }

        self.window_line_counter += 1;
    }

    fn draw_objects_frame_buffer(&self, memory:&dyn ReadOnlyVideoMemory, line:&mut [Color;SCREEN_WIDTH]){
        if !self.sprite_enable{
            return;
        }

        let currrent_line = self.current_line_drawn;

        let mut obj_attributes = Vec::with_capacity(OBJ_PER_LINE as usize);

        for i in (0..OAM_SIZE).step_by(4){
            if obj_attributes.len() >= OBJ_PER_LINE{
                break;
            }
            
            let end_y = memory.read(OAM_ADDRESS + i);
            let end_x = memory.read(OAM_ADDRESS + i + 1);
            let start_y = cmp::max(0, (end_y as i16) - SPRITE_MAX_HEIGHT as i16) as u8;

             //cheks if this sprite apears in this line
             if currrent_line >= end_y || currrent_line < start_y || end_x == 0 || end_x >=168{
                continue;
            }

            //end_y is is the upper y value of the sprite + 16 lines and normal sprite is 8  lines.
            //so checking if this sprite shouldnt be drawn
            //on extended sprite end_y should be within all the values of current line
            if !self.sprite_extended && end_y - currrent_line <= 8{
                continue;
            }
            
            let tile_number = memory.read(OAM_ADDRESS + i + 2);
            let attributes = memory.read(OAM_ADDRESS + i + 3);

            obj_attributes.push(SpriteAttribute::new(end_y, end_x, tile_number, attributes));
        }

        //sprites that occurs first in the oam memory draws first so im reversing it so the first ones will be last and will 
        //draw onto the last ones.
        obj_attributes.reverse();
        //ordering this from the less priority to the higher where the smaller x the priority higher.
        obj_attributes.sort_by(|a, b| b.x.cmp(&a.x));

        for obj_attribute in &obj_attributes{
            let mut sprite = Self::get_sprite(obj_attribute.tile_number, memory, 0x8000, self.sprite_extended);

            if obj_attribute.flip_y {
                sprite.flip_y();
            }
            if obj_attribute.flip_x{
                sprite.flip_x();
            }   

            let end_x = obj_attribute.x;
            let start_x = cmp::max(0, (end_x as i16) - SPRITE_WIDTH as i16) as u8;

            let start_y = cmp::max(0, (obj_attribute.y as i16) - SPRITE_MAX_HEIGHT as i16) as u8;
            let sprite_line = currrent_line - start_y;

            for x in start_x..end_x{
                let pixel = sprite.get_pixel(sprite_line * SPRITE_WIDTH + (x - start_x));
                let color = self.get_obj_color(pixel, obj_attribute.palette_number);
                
                if let Some(c) = color{
                    if !(obj_attribute.is_bg_priority && self.get_bg_color(0) != line[x as usize]){
                        line[x as usize] = c
                    }
                }
            }
        }
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
    
    fn get_sprite(mut index:u8, memory:&dyn ReadOnlyVideoMemory, data_address:u16, extended:bool)->Box<dyn Sprite>{
        let mut sprite:Box<dyn Sprite>;
        if extended{
            //ignore bit 0
            index = (index >> 1) << 1;
            sprite =  Box::new(ExtendedSprite::new());
        }
        else{
            sprite =  Box::new(NormalSprite::new());
        }

        let mut line_number = 0;
        let start:u16 = index as u16 * SPRITE_SIZE_IN_MEMORY;
        let end:u16 = start + ((sprite.size() as u16) *2);
        let raw = Box::into_raw(sprite);
        for j in (start .. end).step_by(2) {
            Self::get_line(memory, raw, data_address + j, line_number);
            line_number += 1;
        }
        unsafe{sprite = Box::from_raw(raw);}

        return sprite;
    }

    fn get_line(memory:&dyn ReadOnlyVideoMemory, sprite:*mut dyn Sprite, address:u16, line_number:u8){
        let byte = memory.read(address);
        let next = memory.read(address + 1);
        for k in (0..SPRITE_WIDTH).rev() {
            let mask = 1 << k;
            let mut value = (byte & mask) >> k;
            value |= ((next & mask) >> k) << 1;
            let swaped = SPRITE_WIDTH - 1 - k;
            unsafe{(*sprite).set_pixel(line_number * SPRITE_WIDTH + swaped, value);}
        }
    }
}
