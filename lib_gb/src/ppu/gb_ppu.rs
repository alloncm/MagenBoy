use crate::mmu::memory::UnprotectedMemory;
use super::ppu_state::PpuState;
use super::color::Color;
use super::colors::*;
use crate::utils::vec2::Vec2;
use super::colors::WHITE;
use super::normal_sprite::NormalSprite;
use super::extended_sprite::ExtendedSprite;
use super::sprite::Sprite;
use super::sprite_attribute::SpriteAttribute;
use crate::utils::{
    memory_registers::*,
    bit_masks::*
};
use std::cmp;

pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_WIDTH: usize = 160;
//CPU frequrncy: 1,048,326 / 60 
pub const CYCLES_PER_FRAME:u32 = 17556;

const OAM_CLOCKS:u8 = 20;
const PIXEL_TRANSFER_CLOCKS:u8 = 43;
const H_BLANK_CLOCKS:u8 = 51;
const DRAWING_CYCLE_CLOCKS: u8 = OAM_CLOCKS + H_BLANK_CLOCKS + PIXEL_TRANSFER_CLOCKS;
const LY_MAX_VALUE:u8 = 153;
const OAM_ADDRESS:u16 = 0xFE00;
const OAM_SIZE:u16 = 0xA0;
const OBJ_PER_LINE:usize = 10;
const SPRITE_WIDTH:u8 = 8;
const NORMAL_SPRITE_HIEGHT:u8 = 8;
const SPRITE_MAX_HEIGHT:u8 = 16;
const BG_SPRITES_PER_LINE:u16 = 32;
const SPRITE_SIZE_IN_MEMORY:u16 = 16;

const BLANK_SCREEN_BUFFER:[u32; SCREEN_HEIGHT * SCREEN_WIDTH] = [GbPpu::color_as_uint(&WHITE);SCREEN_HEIGHT * SCREEN_WIDTH];

pub struct GbPpu {
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

    //interrupts
    pub v_blank_interrupt_request:bool,
    pub h_blank_interrupt_request:bool,
    pub oam_search_interrupt_request:bool,
    pub coincidence_interrupt_request:bool,

    window_active:bool,
    window_line_counter:u8,
    line_rendered:bool,
    current_cycle:u32,
    last_screen_state:bool,
    v_blank_triggered:bool,
    stat_triggered:bool
}

impl Default for GbPpu {
    fn default() -> Self {
        GbPpu {
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
            window_line_counter:0,
            window_active:false,
            current_cycle:0,
            last_screen_state:true,
            v_blank_triggered:false,
            stat_triggered:false,
            //interrupts
            v_blank_interrupt_request:false,
            h_blank_interrupt_request:false,
            oam_search_interrupt_request:false,
            coincidence_interrupt_request:false
        }
    }
}

impl GbPpu {
    const fn color_as_uint(color: &Color) -> u32 {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }

    pub fn get_frame_buffer(&self)->&[u32;SCREEN_HEIGHT*SCREEN_WIDTH]{
        return &self.screen_buffer;
    }

    fn update_ly(&mut self, memory:&mut impl UnprotectedMemory){
        
        let line = self.current_cycle/DRAWING_CYCLE_CLOCKS as u32;
        if self.current_cycle >= CYCLES_PER_FRAME {
            self.current_line_drawn = 0;
            self.line_rendered = false;
            self.current_cycle -= CYCLES_PER_FRAME;
            self.window_line_counter = 0;
            self.window_active = false;
        }
        else if self.current_line_drawn != line as u8{
            self.current_line_drawn = line as u8;
            self.line_rendered = false;
        }
        else if self.current_line_drawn > LY_MAX_VALUE{
            std::panic!("invalid LY register value: {}", self.current_line_drawn);
        }
    }

    fn update_ly_register(&mut self, memory:&mut impl UnprotectedMemory){
        if self.current_line_drawn >= SCREEN_HEIGHT as u8 && !self.v_blank_triggered{
            let mut if_register = memory.read_unprotected(IF_REGISTER_ADDRESS);
            if_register |= BIT_0_MASK;
            memory.write_unprotected(IF_REGISTER_ADDRESS, if_register);
            
            self.v_blank_triggered = true;
        }
        else if self.current_line_drawn < SCREEN_HEIGHT as u8{
            self.v_blank_triggered = false;
        }

        memory.write_unprotected(LY_REGISTER_ADDRESS, self.current_line_drawn);
    }

    fn update_stat_register(&mut self, memory: &mut impl UnprotectedMemory){
        let mut register = memory.read_unprotected(STAT_REGISTER_ADDRESS);
        let mut lcd_stat_interrupt:bool = false;

        if self.current_line_drawn == memory.read_unprotected(LYC_REGISTER_ADDRESS){
            register |= BIT_2_MASK;
            if self.coincidence_interrupt_request {
                lcd_stat_interrupt = true;
            }
        }
        else{
            register &= !BIT_2_MASK;
        }
        
        //clears the 2 lower bits
        register = (register >> 2)<<2;
        register |= self.state as u8;

        match self.state{
            PpuState::OamSearch=>{
                if self.oam_search_interrupt_request{
                    lcd_stat_interrupt = true;
                }
            },
            PpuState::Hblank=>{
                if self.h_blank_interrupt_request{
                    lcd_stat_interrupt = true;
                }
            },
            PpuState::Vblank=>{
                if self.v_blank_interrupt_request{
                    lcd_stat_interrupt = true;
                }
            },
            _=>{}
        }

        if lcd_stat_interrupt{
            if !self.stat_triggered{
                let mut if_register = memory.read_unprotected(IF_REGISTER_ADDRESS);
                if_register |= BIT_1_MASK;
                memory.write_unprotected(IF_REGISTER_ADDRESS, if_register);
                
                self.stat_triggered = true;
            }
        }
        else{
            self.stat_triggered = false;
        }
        
        memory.write_unprotected(STAT_REGISTER_ADDRESS, register);
    }

    fn get_ppu_state(cycle_counter:u32, last_ly:u8)->PpuState{
        if last_ly >= SCREEN_HEIGHT as u8{
            return PpuState::Vblank;
        }

        //getting the reminder of the clocks 
        let current_line_clocks = cycle_counter % DRAWING_CYCLE_CLOCKS as u32;
        
        const OAM_SERACH_END:u8 = OAM_CLOCKS - 1;
        const PIXEL_TRANSFER_START:u8 = OAM_CLOCKS;
        const PIXEL_TRANSFER_END:u8 = OAM_CLOCKS + PIXEL_TRANSFER_CLOCKS - 1;
        const H_BLANK_START:u8 = OAM_CLOCKS + PIXEL_TRANSFER_CLOCKS;
        const H_BLANK_END:u8 = H_BLANK_START + H_BLANK_CLOCKS - 1;

        return match current_line_clocks as u8{
            0 ..= OAM_SERACH_END => PpuState::OamSearch, // 0-19 (20)
            PIXEL_TRANSFER_START ..= PIXEL_TRANSFER_END => PpuState::PixelTransfer, //20-62 (43)
            H_BLANK_START ..= H_BLANK_END => PpuState::Hblank,//63-113(51)
            _=>std::panic!("Error calculating ppu state")
        };
    }

    pub fn update_gb_screen(&mut self, memory: &mut impl UnprotectedMemory, cycles_passed:u32){
        if !self.screen_enable && self.last_screen_state {
            self.current_line_drawn = 0;
            self.current_cycle = 0;
            self.screen_buffer = BLANK_SCREEN_BUFFER;
            self.state = PpuState::Hblank;
            self.window_active = false;
            self.last_screen_state = self.screen_enable;
            return;
        }
        else if !self.screen_enable{
            return;
        }
        
        self.last_screen_state = self.screen_enable;

        self.current_cycle += cycles_passed as u32;
        self.update_ly(memory);
        self.state = Self::get_ppu_state(self.current_cycle, self.current_line_drawn);
        
        self.update_ly_register(memory);
        self.update_stat_register(memory);

        if self.state as u8 == PpuState::PixelTransfer as u8{
            if !self.line_rendered {
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
    }

    fn get_bg_frame_buffer(&self, memory: &impl UnprotectedMemory)-> [Color;SCREEN_WIDTH] {
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
                let chr: u8 = memory.read_unprotected(address + (index*BG_SPRITES_PER_LINE) + i);
                let sprite = Self::get_normal_sprite(chr, memory, 0x8000);
                line_sprites.push(sprite);
            }
        } 
        else {
            for i in 0..BG_SPRITES_PER_LINE {
                let mut chr: u8 = memory.read_unprotected(address + (index*BG_SPRITES_PER_LINE) + i);
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

    fn get_normal_sprite(index:u8, memory:&impl UnprotectedMemory, data_address:u16)->NormalSprite{
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

    
    fn draw_window_frame_buffer(&mut self, memory: &impl UnprotectedMemory, line:&mut [Color;SCREEN_WIDTH]) {
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
                let chr: u8 = memory.read_unprotected(address + (index*BG_SPRITES_PER_LINE) + i);
                let sprite = Self::get_normal_sprite(chr, memory, 0x8000);
                line_sprites.push(sprite);
            }
        } 
        else {
            for i in 0..BG_SPRITES_PER_LINE {
                let mut chr: u8 = memory.read_unprotected(address + (index*BG_SPRITES_PER_LINE) + i);
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

    fn draw_objects_frame_buffer(&self, memory:&impl UnprotectedMemory, line:&mut [Color;SCREEN_WIDTH]){
        if !self.sprite_enable{
            return;
        }

        let currrent_line = self.current_line_drawn;

        let mut obj_attributes = Vec::with_capacity(OBJ_PER_LINE as usize);

        for i in (0..OAM_SIZE).step_by(4){
            if obj_attributes.len() >= OBJ_PER_LINE{
                break;
            }
            
            let end_y = memory.read_unprotected(OAM_ADDRESS + i);
            let end_x = memory.read_unprotected(OAM_ADDRESS + i + 1);
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
            
            let tile_number = memory.read_unprotected(OAM_ADDRESS + i + 2);
            let attributes = memory.read_unprotected(OAM_ADDRESS + i + 3);

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

            let end_x = cmp::min(obj_attribute.x, SCREEN_WIDTH as u8);
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
    
    fn get_sprite(mut index:u8, memory:&impl UnprotectedMemory, data_address:u16, extended:bool)->Box<dyn Sprite>{
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

    fn get_line(memory:&impl UnprotectedMemory, sprite:*mut dyn Sprite, address:u16, line_number:u8){
        let byte = memory.read_unprotected(address);
        let next = memory.read_unprotected(address + 1);
        for k in (0..SPRITE_WIDTH).rev() {
            let mask = 1 << k;
            let mut value = (byte & mask) >> k;
            value |= ((next & mask) >> k) << 1;
            let swaped = SPRITE_WIDTH - 1 - k;
            unsafe{(*sprite).set_pixel(line_number * SPRITE_WIDTH + swaped, value);}
        }
    }
}
