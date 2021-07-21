use crate::utils::{vec2::Vec2, bit_masks::*};
use crate::mmu::vram::VRam;
use super::{ppu_state::PpuState, sprite_attribute::SpriteAttribute};

enum FethcingState{
    TileNumber,
    LowTileData(u8),
    HighTileData(u8,u8),
    Push(u8,u8)
}

pub struct FifoPpu{
    oam_entries:[SpriteAttribute; 10],
    vram: VRam,
    current_oam_entry:u8,
    t_cycles_passed:u16,
    state:PpuState,
    lcd_control:u8,
    ly_register:u8,
    window_pos:Vec2<u8>,
    bg_pos:Vec2<u8>,
    pixel_fething_state: FethcingState,

    pos_counter: Vec2<u8>,
    bg_fifo: Vec<u8>
}

impl FifoPpu{
    pub fn cycle(&mut self, m_cycles:u8, oam:&[u8;0xA0], extended_sprite:bool)->Vec<u8>{
        let sprite_height = if extended_sprite {16} else {8};

        let mut pixels_to_push_to_lcd = Vec::<u8>::new();

        for _ in 0..m_cycles{
            match self.state{
               PpuState::OamSearch=>{
                    for _ in 0..2{
                        self.t_cycles_passed += 2; //half a m_cycle
                        let oam_index = self.t_cycles_passed / 2;
                        let oam_entry_address = (oam_index * 4) as usize;
                        let end_y = oam[oam_entry_address];
                        let end_x = oam[oam_entry_address + 1];
                    
                        if end_x > 0 && self.ly_register + 16 >= end_y && self.ly_register + 16 < end_y + sprite_height && self.current_oam_entry < 10{
                            let tile_number = oam[oam_entry_address + 2];
                            let attributes = oam[oam_entry_address + 3];
                            self.oam_entries[self.current_oam_entry as usize] = SpriteAttribute::new(end_y, end_x, tile_number, attributes);
                            self.current_oam_entry += 1;
                        }
                    }

                    if self.t_cycles_passed == 80{
                        self.state = PpuState::PixelTransfer;
                    }
                }
                PpuState::Hblank=>{
                    self.t_cycles_passed += 4;
                    if self.t_cycles_passed == 456{
                        if self.ly_register == 143{
                            self.state = PpuState::Vblank;
                        }
                        else{
                            self.state = PpuState::OamSearch;
                        }
                        self.t_cycles_passed = 0;
                        self.ly_register += 1;
                    }
                }
                PpuState::Vblank=>{
                    self.t_cycles_passed += 4;
                    if self.t_cycles_passed == 4560{
                        self.state = PpuState::OamSearch;
                        self.t_cycles_passed = 0;
                        self.ly_register = 0;
                    }
                    else{
                        self.ly_register = 144 + (self.t_cycles_passed % 456) as u8;
                    }
                }
                PpuState::PixelTransfer=>{
                    match self.pixel_fething_state{
                        FethcingState::TileNumber=>{
                            let tile_num = if self.is_redering_wnd(){
                                let tile_map_address:u16 = if self.lcd_control & BIT_6_MASK == 0 {0x1800} else {0x1C00};
                                self.vram.read_current_bank(tile_map_address + ((32 * (self.pos_counter. y / 8)) + (self.pos_counter.x / 8) )as u16)
                            }
                            else{
                                let tile_map_address = if self.lcd_control & BIT_3_MASK == 0 {0x1800} else {0x1C00};
                                let scx_offset = self.bg_pos.x as u16 / 8;
                                let scy_offset = (((self.bg_pos.y + self.pos_counter.y) & 0xFF) / 8) as u16;
                                self.vram.read_current_bank(tile_map_address + (32 * scy_offset + (self.pos_counter.x as u16 + scx_offset) & 31))
                            };

                            self.pixel_fething_state = FethcingState::LowTileData(tile_num);
                            self.t_cycles_passed += 2;
                        }
                        FethcingState::LowTileData(tile_num)=>{
                            let current_tile_base_data_address = if self.lcd_control & BIT_4_MASK == 0 && tile_num & BIT_7_MASK == 0 {0x1000} else {0};
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
                            let current_tile_base_data_address = if self.lcd_control & BIT_4_MASK == 0 && tile_num & BIT_7_MASK == 0 {0x1000} else {0};
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
                                pixel |= (high_data & mask) >> (i - 1);
                                self.bg_fifo.push(pixel);
                            }

                            self.pixel_fething_state = FethcingState::TileNumber;
                            self.t_cycles_passed += 2;
                        }
                    }
                }
            }
            
            pixels_to_push_to_lcd.append(&mut self.bg_fifo);
        }   
        
        return pixels_to_push_to_lcd;
    }

    fn is_redering_wnd(&self)->bool{
        self.window_pos.x >= self.bg_pos.x && self.window_pos.y >= self.bg_pos.y && self.lcd_control & BIT_5_MASK != 0
    }
}

struct PixelFetcher{

}

impl PixelFetcher{
    pub fn cycle(&mut self){

    }
}