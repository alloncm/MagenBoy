use crate::{mmu::vram::VRam, utils::{bit_masks::*, vec2::Vec2}};

use super::fetching_state::FethcingState;

pub struct BGFetcher{
    pub fifo:Vec<u8>,
    pub window_line_counter:u8,

    current_x_pos:u8,
    rendered_window:bool,
    rendering_window:bool,
    current_fetching_state:FethcingState,
}

impl BGFetcher{
    pub fn new()->Self{
        BGFetcher{
            current_fetching_state:FethcingState::TileNumber,
            current_x_pos:0,
            fifo:Vec::<u8>::with_capacity(8),
            window_line_counter:0,
            rendered_window:false,
            rendering_window:false
        }
    }

    pub fn reset(&mut self){
        self.fifo.clear();
        self.current_x_pos = 0;
        self.current_fetching_state = FethcingState::TileNumber;
        self.rendered_window = false;
        self.rendering_window = false;
    }

    pub fn try_increment_window_counter(&mut self){
        if self.rendered_window{
            self.window_line_counter += 1;
        }
    }

    pub fn fetch_pixels(&mut self, vram:&VRam, lcd_control:u8, ly_register:u8, window_pos:&Vec2<u8>, bg_pos:&Vec2<u8>){
        let last_rendering_status = self.rendering_window;
        self.rendering_window = self.is_redering_wnd(lcd_control, window_pos, ly_register);
        if self.rendering_window{
            self.rendered_window = true;

            // In case I was rendering a background pixel need to reset the state of the fectcher 
            // (and maybe clear the fifo but right now Im not doing it since im not sure what about the current_x_pos var)
            if !last_rendering_status{
                self.current_fetching_state = FethcingState::TileNumber;
            }
        }

        match self.current_fetching_state{
            FethcingState::TileNumber=>{
                let tile_num = if self.rendering_window{
                    let tile_map_address:u16 = if (lcd_control & BIT_6_MASK) == 0 {0x1800} else {0x1C00};
                    vram.read_current_bank(tile_map_address + (32 * (self.window_line_counter as u16 / 8)) + ((self.current_x_pos - window_pos.x) as u16 / 8))
                }
                else{
                    let tile_map_address = if (lcd_control & BIT_3_MASK) == 0 {0x1800} else {0x1C00};
                    let scx_offset = ((bg_pos.x as u16 + self.current_x_pos as u16) / 8 ) & 31;
                    let scy_offset = ((bg_pos.y as u16 + ly_register as u16) & 0xFF) / 8;

                    vram.read_current_bank(tile_map_address + ((32 * scy_offset) + scx_offset))
                };

                self.current_fetching_state = FethcingState::LowTileData(tile_num);
            }
            FethcingState::LowTileData(tile_num)=>{
                let current_tile_base_data_address = if (lcd_control & BIT_4_MASK) == 0 && (tile_num & BIT_7_MASK) == 0 {0x1000} else {0};
                let current_tile_data_address = current_tile_base_data_address + (tile_num  as u16 * 16);
                let low_data = if self.rendering_window{
                    vram.read_current_bank(current_tile_data_address + (2 * (self.window_line_counter % 8)) as u16)
                } else{
                    vram.read_current_bank(current_tile_data_address + (2 * ((bg_pos.y as u16 + ly_register as u16) % 8)) )
                };

                self.current_fetching_state = FethcingState::HighTileData(tile_num, low_data);
            }
            FethcingState::HighTileData(tile_num, low_data)=>{
                let current_tile_base_data_address = if (lcd_control & BIT_4_MASK) == 0 && (tile_num & BIT_7_MASK) == 0 {0x1000} else {0};
                let current_tile_data_address = current_tile_base_data_address + (tile_num  as u16 * 16);
                let high_data = if self.rendering_window{
                    vram.read_current_bank(current_tile_data_address + (2 * (self.window_line_counter % 8)) as u16 + 1)
                } else{
                    vram.read_current_bank(current_tile_data_address + (2 * ((bg_pos.y as u16 + ly_register as u16) % 8)) + 1)
                };

                self.current_fetching_state = FethcingState::Push(low_data, high_data);
            }
            FethcingState::Push(low_data, high_data)=>{
                if self.fifo.is_empty(){
                    if lcd_control & BIT_0_MASK == 0{
                        for _ in 0..8{
                            self.fifo.push(0);
                            self.current_x_pos += 1;
                        }
                    }
                    else{
                        for i in (0..8).rev(){
                            let mask = 1 << i;
                            let mut pixel = (low_data & mask) >> i;
                            pixel |= ((high_data & mask) >> i) << 1;
                            self.fifo.push(pixel);
                            self.current_x_pos += 1;
                        }
                    }

                    self.current_fetching_state = FethcingState::TileNumber;
                }
            }
        }
    }

    fn is_redering_wnd(&self, lcd_control:u8, window_pos:&Vec2<u8>, ly_register:u8)->bool{
        window_pos.x <= self.current_x_pos && window_pos.y <= ly_register && (lcd_control & BIT_5_MASK) != 0
    }
}