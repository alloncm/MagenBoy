use std::mem::{self, MaybeUninit};

use crate::utils::{vec2::Vec2, bit_masks::*};
use crate::mmu::vram::VRam;
use super::color::Color;
use super::colors::*;
use super::gfx_device::GfxDevice;
use super::{ppu_state::PpuState, sprite_attribute::SpriteAttribute};

enum FethcingState{
    TileNumber,
    LowTileData(u8),
    HighTileData(u8,u8),
    Push(u8,u8)
}

pub struct FifoPpu<GFX: GfxDevice>{
    gfx_device: GFX,

    oam_entries:[SpriteAttribute; 10],
    pub vram: VRam,
    pub oam:[u8;0xA0],
    current_oam_entry:u8,
    t_cycles_passed:u16,
    pub state:PpuState,
    pub lcd_control:u8,
    pub stat_register:u8,
    pub lyc_register:u8,
    pub ly_register:u8,
    pub window_pos:Vec2<u8>,
    pub bg_pos:Vec2<u8>,
    pixel_fething_state: FethcingState,
    pub bg_color_mapping: [Color; 4],

    screen_buffer: [u32; 160*144],
    screen_buffer_index:usize,

    //interrupts
    pub v_blank_interrupt_request:bool,
    pub h_blank_interrupt_request:bool,
    pub oam_search_interrupt_request:bool,
    pub coincidence_interrupt_request:bool,

    pos_counter: Vec2<u8>,
    bg_fifo: Vec<u8>,
}

impl<GFX:GfxDevice> FifoPpu<GFX>{

    pub fn new(device:GFX) -> Self {
        let oam_entries = {
            let mut data: [MaybeUninit<SpriteAttribute>; 10] = unsafe{
                MaybeUninit::uninit().assume_init()
            };

            for elem in &mut data[..]{
                *elem = MaybeUninit::new(SpriteAttribute::new(0, 0, 0, 0));
            }

            unsafe{mem::transmute::<_, [SpriteAttribute;10]>(data)}
        };

        Self{
            gfx_device: device,
            vram: VRam::default(),
            oam: [0;0xA0],
            stat_register: 0,
            lyc_register: 0,
            lcd_control: 0,
            bg_pos: Vec2::<u8>{x:0, y:0},
            window_pos: Vec2::<u8>{x:0,y:0},
            screen_buffer:[0;160*144],
            bg_color_mapping:[WHITE, LIGHT_GRAY, DARK_GRAY, BLACK],
            ly_register:0,
            state: PpuState::OamSearch,
            pos_counter: Vec2::<u8>{x:0,y:0},
            //interrupts
            v_blank_interrupt_request:false, 
            h_blank_interrupt_request:false,
            oam_search_interrupt_request:false, 
            coincidence_interrupt_request:false,
            oam_entries:oam_entries,
            current_oam_entry:0,
            pixel_fething_state:FethcingState::TileNumber,
            screen_buffer_index:0, 
            t_cycles_passed:0,
            bg_fifo:Vec::<u8>::with_capacity(16),
        }
    }

    pub fn turn_off(&mut self){
        self.screen_buffer_index = 0;
        self.t_cycles_passed = 0;
        unsafe{
            std::ptr::write_bytes(self.screen_buffer.as_mut_ptr(), 0xFF, self.screen_buffer.len());
        }
        self.gfx_device.swap_buffer(&self.screen_buffer);
        self.state = PpuState::Hblank;
        self.bg_fifo.clear();
        self.ly_register = 0;

    }

    pub fn turn_on(&mut self){
        self.state = PpuState::OamSearch;
    }

    pub fn cycle(&mut self, m_cycles:u8, if_register:&mut u8){
        if self.lcd_control & BIT_7_MASK == 0{
            return;
        }

        let pixels = self.cycle_fetcher(m_cycles, if_register);

        //update stat register
        self.stat_register &= 0b1111_1100; //clear first 2 bits
        self.stat_register |= self.state as u8;

        for pixel in pixels.as_slice(){
            let p = self.bg_color_mapping[*pixel as usize].clone();
            self.screen_buffer[self.screen_buffer_index] = Self::color_as_uint(&p);
            self.screen_buffer_index += 1;
            if self.screen_buffer_index == self.screen_buffer.len(){
                self.gfx_device.swap_buffer(&self.screen_buffer);
                self.screen_buffer_index = 0;
            }
        }
    }

    fn cycle_fetcher(&mut self, m_cycles:u8, if_register:&mut u8)->Vec<u8>{
        let sprite_height = if (self.lcd_control & BIT_2_MASK) != 0 {16} else {8};

        let mut pixels_to_push_to_lcd = Vec::<u8>::new();

        for _ in 0..m_cycles{
            match self.state{
                PpuState::OamSearch=>{
                    let oam_index = self.t_cycles_passed / 2;
                    let oam_entry_address = (oam_index * 4) as usize;
                    let end_y = self.oam[oam_entry_address];
                    let end_x = self.oam[oam_entry_address + 1];
                
                    if end_x > 0 && self.ly_register + 16 >= end_y && self.ly_register + 16 < end_y + sprite_height && self.current_oam_entry < 10{
                        let tile_number = self.oam[oam_entry_address + 2];
                        let attributes = self.oam[oam_entry_address + 3];
                        self.oam_entries[self.current_oam_entry as usize] = SpriteAttribute::new(end_y, end_x, tile_number, attributes);
                        self.current_oam_entry += 1;
                    }
                    
                    self.t_cycles_passed += 2; //half a m_cycle
                    
                    if self.t_cycles_passed == 80{
                        self.state = PpuState::PixelTransfer;
                        self.pixel_fething_state = FethcingState::TileNumber;
                    }
                }
                PpuState::Hblank=>{
                    if self.t_cycles_passed == 456{
                        if self.ly_register == 143{
                            self.state = PpuState::Vblank;
                            if self.v_blank_interrupt_request{
                                *if_register |= BIT_1_MASK;
                            }
                        }
                        else{
                            self.state = PpuState::OamSearch;
                            if self.oam_search_interrupt_request{
                                *if_register |= BIT_1_MASK;
                            }
                        }
                        self.t_cycles_passed = 0;
                        self.ly_register += 1;
                    }
                    
                    self.t_cycles_passed += 2;
                }
                PpuState::Vblank=>{
                    if self.t_cycles_passed == 4560{
                        self.state = PpuState::OamSearch;
                        if self.oam_search_interrupt_request{
                            *if_register |= BIT_1_MASK;
                        }
                        self.t_cycles_passed = 0;
                        self.ly_register = 0;
                    }
                    else{
                        self.ly_register = 144 + (self.t_cycles_passed / 456) as u8;
                    }
                    
                    self.t_cycles_passed += 2;
                }
                PpuState::PixelTransfer=>{
                    for _ in 0..2{
                        match self.pixel_fething_state{
                            FethcingState::TileNumber=>{
                                let tile_num = if self.is_redering_wnd(){
                                    let tile_map_address:u16 = if (self.lcd_control & BIT_6_MASK) == 0 {0x1800} else {0x1C00};
                                    self.vram.read_current_bank(tile_map_address + ((32 * (self.pos_counter. y / 8)) + (self.pos_counter.x / 8) )as u16)
                                }
                                else{
                                    let tile_map_address = if (self.lcd_control & BIT_3_MASK) == 0 {0x1800} else {0x1C00};
                                    let scx_offset = ((self.bg_pos.x as u16 + self.pos_counter.x as u16) / 8 ) & 31;
                                    let scy_offset = ((self.bg_pos.y as u16 + self.ly_register as u16) & 0xFF) / 8;

                                    self.vram.read_current_bank(tile_map_address + ((32 * scy_offset) + scx_offset))
                                };

                                self.pixel_fething_state = FethcingState::LowTileData(tile_num);
                                self.t_cycles_passed += 2;
                            }
                            FethcingState::LowTileData(tile_num)=>{
                                let current_tile_base_data_address = if (self.lcd_control & BIT_4_MASK) == 0 && (tile_num & BIT_7_MASK) == 0 {0x1000} else {0};
                                let current_tile_data_address = current_tile_base_data_address + (tile_num  as u16 * 16);
                                let low_data = if self.is_redering_wnd(){
                                    self.vram.read_current_bank(current_tile_data_address + (2 * (self.pos_counter.y % 8)) as u16)
                                } else{
                                    self.vram.read_current_bank(current_tile_data_address + (2 * ((self.bg_pos.y + self.ly_register) % 8)) as u16)
                                };

                                self.pixel_fething_state = FethcingState::HighTileData(tile_num, low_data);
                                self.t_cycles_passed += 2;
                            }
                            FethcingState::HighTileData(tile_num, low_data)=>{
                                let current_tile_base_data_address = if (self.lcd_control & BIT_4_MASK) == 0 && (tile_num & BIT_7_MASK) == 0 {0x1000} else {0};
                                let current_tile_data_address = current_tile_base_data_address + (tile_num  as u16 * 16);
                                let high_data = if self.is_redering_wnd(){
                                    self.vram.read_current_bank(current_tile_data_address + (2 * (self.pos_counter.y % 8)) as u16 + 1)
                                } else{
                                    self.vram.read_current_bank(current_tile_data_address + (2 * ((self.bg_pos.y + self.ly_register) % 8)) as u16 + 1)
                                };

                                self.pixel_fething_state = FethcingState::Push(low_data, high_data);
                                self.t_cycles_passed += 2;
                            }
                            FethcingState::Push(low_data, high_data)=>{
                                for i in (0..8).rev(){
                                    let mask = 1 << i;
                                    let mut pixel = (low_data & mask) >> i;
                                    pixel |= ((high_data & mask) >> i) << 1;
                                    self.bg_fifo.push(pixel);
                                }

                                self.pixel_fething_state = FethcingState::TileNumber;
                                self.t_cycles_passed += 2;
                            
                                self.pos_counter.x += 8;
                            }
                        }

                        if self.pos_counter.x == 160{
                            self.state = PpuState::Hblank;
                            if self.h_blank_interrupt_request{
                                *if_register |= BIT_1_MASK;
                            }
                            self.pos_counter.x = 0;
                        }
                    }
                }
            }

            if self.ly_register == self.lyc_register{
                self.stat_register |= BIT_2_MASK;
                if self.coincidence_interrupt_request{
                    *if_register |= BIT_1_MASK;
                }
            }
            else{
                self.stat_register &= !BIT_2_MASK;
            }
            
            pixels_to_push_to_lcd.append(&mut self.bg_fifo);
        }   
        
        return pixels_to_push_to_lcd;
    }

    fn is_redering_wnd(&self)->bool{
        self.window_pos.x >= self.bg_pos.x && self.window_pos.y >= self.bg_pos.y && (self.lcd_control & BIT_5_MASK) != 0
    }

    fn color_as_uint(color: &Color) -> u32 {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }
}    
