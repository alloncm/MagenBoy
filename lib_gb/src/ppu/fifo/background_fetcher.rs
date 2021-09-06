use crate::{mmu::vram::VRam, utils::{bit_masks::*, vec2::Vec2}};
use super::{fetcher_state_machine::FetcherStateMachine, fetching_state::*};

pub struct BackgroundFetcher{
    pub fifo:Vec<u8>,
    pub window_line_counter:u8,

    current_x_pos:u8,
    rendered_window:bool,
    rendering_window:bool,
    fetcher_state_machine:FetcherStateMachine,
}

impl BackgroundFetcher{
    pub fn new()->Self{
        let state_machine = [FetchingState::Sleep, FetchingState::FetchTileNumber, FetchingState::Sleep, FetchingState::FetchLowTile, FetchingState::Sleep, FetchingState::FetchHighTile, FetchingState::Sleep, FetchingState::Push];
        BackgroundFetcher{
            fetcher_state_machine:FetcherStateMachine::new(state_machine),
            current_x_pos:0,
            fifo:Vec::<u8>::with_capacity(8),
            window_line_counter:0,
            rendered_window:false,
            rendering_window:false,
        }
    }

    pub fn reset(&mut self){
        self.fifo.clear();
        self.current_x_pos = 0;
        self.fetcher_state_machine.reset();
        self.rendered_window = false;
        self.rendering_window = false;
    }

    pub fn pause(&mut self){
        self.fetcher_state_machine.reset();
    }

    pub fn try_increment_window_counter(&mut self){
        if self.rendered_window{
            self.window_line_counter += 1;
        }
    }

    pub fn fetch_pixels(&mut self, vram:&VRam, lcd_control:u8, ly_register:u8, window_pos:&Vec2<u8>, bg_pos:&Vec2<u8>){
        let last_rendering_status = self.rendering_window;
        self.rendering_window = self.is_rendering_wnd(lcd_control, window_pos, ly_register);
        if self.rendering_window{
            self.rendered_window = true;

            // In case I was rendering a background pixel need to reset the state of the fectcher 
            // (and maybe clear the fifo but right now Im not doing it since im not sure what about the current_x_pos var)
            if !last_rendering_status{
                self.fetcher_state_machine.reset();
            }
        }

        match self.fetcher_state_machine.current_state(){
            FetchingState::FetchTileNumber=>{
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

                self.fetcher_state_machine.data.reset();
                self.fetcher_state_machine.data.tile_data = Some(tile_num);
            }
            FetchingState::FetchLowTile=>{
                let tile_num = self.fetcher_state_machine.data.tile_data.expect("State machine is corrupted, No Tile data on FetchLowTIle");
                let address = self.get_tila_data_address(lcd_control, bg_pos, ly_register, tile_num);
                let low_data = vram.read_current_bank(address);

                self.fetcher_state_machine.data.low_tile_data = Some(low_data);
            }
            FetchingState::FetchHighTile=>{
                let tile_num= self.fetcher_state_machine.data.tile_data.expect("State machine is corrupted, No Tile data on FetchHighTIle");
                let address = self.get_tila_data_address(lcd_control, bg_pos, ly_register, tile_num);
                let high_data = vram.read_current_bank(address + 1);

                self.fetcher_state_machine.data.high_tile_data = Some(high_data);
            }
            FetchingState::Push=>{
                let low_data = self.fetcher_state_machine.data.low_tile_data.expect("State machine is corrupted, No Low data on Push");
                let high_data = self.fetcher_state_machine.data.high_tile_data.expect("State machine is corrupted, No High data on Push");
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
                }
            }
            FetchingState::Sleep=>{}
        }

        self.fetcher_state_machine.advance();
    }

    fn get_tila_data_address(&self, lcd_control:u8, bg_pos:&Vec2<u8>, ly_register:u8, tile_num:u8)->u16{
        let current_tile_base_data_address = if (lcd_control & BIT_4_MASK) == 0 && (tile_num & BIT_7_MASK) == 0 {0x1000} else {0};
        let current_tile_data_address = current_tile_base_data_address + (tile_num  as u16 * 16);
        return if self.rendering_window{
            current_tile_data_address + (2 * (self.window_line_counter % 8)) as u16
        } else{
            current_tile_data_address + (2 * ((bg_pos.y as u16 + ly_register as u16) % 8))
        };
    }

    fn is_rendering_wnd(&self, lcd_control:u8, window_pos:&Vec2<u8>, ly_register:u8)->bool{
        window_pos.x <= self.current_x_pos && window_pos.y <= ly_register && (lcd_control & BIT_5_MASK) != 0
    }
}