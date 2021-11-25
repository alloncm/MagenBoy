use crate::{mmu::vram::VRam, utils::{bit_masks::*, fixed_size_queue::FixedSizeQueue, vec2::Vec2}};
use super::{FIFO_SIZE, SPRITE_WIDTH, fetcher_state_machine::FetcherStateMachine, fetching_state::*};

pub struct BackgroundFetcher{
    pub fifo:FixedSizeQueue<u8, FIFO_SIZE>,
    pub window_line_counter:u8,
    pub has_wy_reached_ly:bool,

    current_x_pos:u8,
    rendering_window:bool,
    fetcher_state_machine:FetcherStateMachine,
}

impl BackgroundFetcher{
    pub fn new()->Self{
        let state_machine = [FetchingState::Sleep, FetchingState::FetchTileNumber, FetchingState::Sleep, FetchingState::FetchLowTile, FetchingState::Sleep, FetchingState::FetchHighTile, FetchingState::Sleep, FetchingState::Push];
        BackgroundFetcher{
            fetcher_state_machine:FetcherStateMachine::new(state_machine),
            current_x_pos:0,
            fifo:FixedSizeQueue::<u8, FIFO_SIZE>::new(),
            window_line_counter:0,
            rendering_window:false,
            has_wy_reached_ly:false,
        }
    }

    pub fn reset(&mut self){
        self.fifo.clear();
        self.current_x_pos = 0;
        self.fetcher_state_machine.reset();
        self.rendering_window = false;
    }

    pub fn pause(&mut self){
        self.fetcher_state_machine.reset();
    }

    pub fn try_increment_window_counter(&mut self, ly_register:u8, wy_register:u8){
        if self.rendering_window && ly_register >= wy_register{
            self.window_line_counter += 1;
        }
    }

    pub fn fetch_pixels(&mut self, vram:&VRam, lcd_control:u8, ly_register:u8, window_pos:&Vec2<u8>, bg_pos:&Vec2<u8>){
        self.has_wy_reached_ly = self.has_wy_reached_ly || ly_register == window_pos.y;
        let last_rendering_status = self.rendering_window;
        self.rendering_window = self.is_rendering_wnd(lcd_control, window_pos);
        
        // In case I was rendering a background pixel need to reset the state of the fetcher 
        // (and maybe clear the fifo but right now Im not doing it since im not sure what about the current_x_pos var)
        if self.rendering_window && !last_rendering_status{
            self.fetcher_state_machine.reset();
        }

        match self.fetcher_state_machine.current_state(){
            FetchingState::FetchTileNumber=>{
                let tile_num = if self.rendering_window{
                    let tile_map_address:u16 = if (lcd_control & BIT_6_MASK) == 0 {0x1800} else {0x1C00};
                    vram.read_current_bank(tile_map_address + (32 * (self.window_line_counter as u16 / SPRITE_WIDTH as u16)) + ((self.current_x_pos - window_pos.x) as u16 / SPRITE_WIDTH as u16))
                }
                else{
                    let tile_map_address = if (lcd_control & BIT_3_MASK) == 0 {0x1800} else {0x1C00};
                    let scx_offset = ((bg_pos.x as u16 + self.current_x_pos as u16) / SPRITE_WIDTH as u16 ) & 31;
                    let scy_offset = ((bg_pos.y as u16 + ly_register as u16) & 0xFF) / SPRITE_WIDTH as u16;

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
                if self.fifo.len() == 0{
                    if lcd_control & BIT_0_MASK == 0{
                        for _ in 0..SPRITE_WIDTH{
                            //When the baclkground is off pushes 0
                            self.fifo.push(0);
                            self.current_x_pos += 1;
                        }
                    }
                    else{
                        for i in (0..SPRITE_WIDTH).rev(){
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
            current_tile_data_address + (2 * (self.window_line_counter % SPRITE_WIDTH)) as u16
        } else{
            current_tile_data_address + (2 * ((bg_pos.y as u16 + ly_register as u16) % SPRITE_WIDTH as u16))
        };
    }

    fn is_rendering_wnd(&self, lcd_control:u8, window_pos:&Vec2<u8>)->bool{
        window_pos.x <= self.current_x_pos && self.has_wy_reached_ly && (lcd_control & BIT_5_MASK) != 0
    }
}