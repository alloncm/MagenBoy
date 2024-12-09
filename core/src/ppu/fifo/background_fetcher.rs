use crate::{utils::{bit_masks::*, fixed_size_queue::FixedSizeQueue, vec2::Vec2}, ppu::{VRam, attributes::GbcBackgroundAttributes}};
use super::{FIFO_SIZE, SPRITE_WIDTH, fetching_state::*, get_decoded_pixel};

#[derive(Clone, Copy, Default)]
pub struct BackgroundPixel{pub color_index:u8, pub attributes:GbcBackgroundAttributes}

const EMPTY_FIFO_BUFFER:[BackgroundPixel;FIFO_SIZE] = [BackgroundPixel{color_index:0, attributes:DEFAULT_GBC_ATTRIBUTES};FIFO_SIZE];
const DEFAULT_GBC_ATTRIBUTES: GbcBackgroundAttributes = GbcBackgroundAttributes::new(0);
const TILES_IN_VRAM_ROW:u16 = 32;

pub struct BackgroundFetcher{
    pub fifo:FixedSizeQueue<BackgroundPixel, FIFO_SIZE>,
    pub window_line_counter:u8,
    pub has_wy_reached_ly:bool,
    pub rendering_window:bool,

    current_x_pos:u8,
    fetcher_state_machine:FetcherStateMachine,
    scanline_rendering_started:bool,
    cgb_attribute:GbcBackgroundAttributes,
}

impl BackgroundFetcher{ 
    pub fn new()->Self{
        let state_machine = [FetchingState::Sleep, FetchingState::FetchTileNumber, FetchingState::Sleep, FetchingState::FetchLowTile, FetchingState::Sleep, FetchingState::FetchHighTile, FetchingState::Push, FetchingState::Sleep];
        BackgroundFetcher{
            fetcher_state_machine:FetcherStateMachine::new(state_machine),
            cgb_attribute:DEFAULT_GBC_ATTRIBUTES,
            current_x_pos:0,
            fifo:FixedSizeQueue::<BackgroundPixel, FIFO_SIZE>::new(),
            window_line_counter:0,
            rendering_window:false,
            has_wy_reached_ly:false,
            scanline_rendering_started:false,
        }
    }

    pub fn reset(&mut self){
        self.fifo.clear();
        self.current_x_pos = 0;
        self.fetcher_state_machine.reset();
        self.rendering_window = false;
        self.scanline_rendering_started = false;
    }

    pub fn pause(&mut self){
        self.fetcher_state_machine.reset();
    }

    pub fn try_increment_window_counter(&mut self, ly_register:u8, wy_register:u8){
        if self.rendering_window && ly_register >= wy_register{
            self.window_line_counter += 1;
        }
    }

    pub fn fetch_pixels(&mut self, vram:&VRam, lcd_control:u8, ly_register:u8, window_pos:&Vec2<u8>, bg_pos:&Vec2<u8>, cgb_enabled:bool){
        self.has_wy_reached_ly = self.has_wy_reached_ly || ly_register == window_pos.y;
        self.rendering_window = self.is_rendering_wnd(lcd_control, window_pos);

        match self.fetcher_state_machine.current_state(){
            FetchingState::FetchTileNumber=>{
                let address = if self.rendering_window{
                    let tile_map_address:u16 = if (lcd_control & BIT_6_MASK) == 0 {0x1800} else {0x1C00};
                    tile_map_address + (TILES_IN_VRAM_ROW * (self.window_line_counter as u16 / SPRITE_WIDTH as u16)) + ((self.current_x_pos - window_pos.x) as u16 / SPRITE_WIDTH as u16)
                }
                else{
                    let tile_map_address = if (lcd_control & BIT_3_MASK) == 0 {0x1800} else {0x1C00};
                    let scx_offset = ((bg_pos.x as u16 + self.current_x_pos as u16) / SPRITE_WIDTH as u16 ) & 31;
                    let scy_offset = ((bg_pos.y as u16 + ly_register as u16) & 0xFF) / SPRITE_WIDTH as u16;
                    tile_map_address + ((TILES_IN_VRAM_ROW * scy_offset) + scx_offset)
                };

                self.fetcher_state_machine.data.reset();
                self.cgb_attribute = if cgb_enabled {GbcBackgroundAttributes::new(vram.read_bank(address, 1))} else {DEFAULT_GBC_ATTRIBUTES};
                self.fetcher_state_machine.data.tile_data = vram.read_bank(address, 0);
                // Calculating once per fetching cycle might be inaccurate (not sure), but could improve perf
                self.fetcher_state_machine.data.tile_data_address = self.get_tila_data_address(lcd_control, bg_pos.y, ly_register, self.fetcher_state_machine.data.tile_data, cgb_enabled);
            }
            FetchingState::FetchLowTile=>{
                let bank = if cgb_enabled {self.cgb_attribute.attribute.gbc_bank as u8}else{0};
                let low_data = vram.read_bank(self.fetcher_state_machine.data.tile_data_address, bank);
                self.fetcher_state_machine.data.low_tile_data = low_data;
            }
            FetchingState::FetchHighTile=>{
                let bank = if cgb_enabled {self.cgb_attribute.attribute.gbc_bank as u8}else{0};
                let high_data = vram.read_bank(self.fetcher_state_machine.data.tile_data_address + 1, bank);
                self.fetcher_state_machine.data.high_tile_data = high_data;

                // The gameboy has this quirk that in the first fetch of the scanline it reset itself after reaching the fetch high tile step
                if !self.scanline_rendering_started{
                    self.reset();
                    self.scanline_rendering_started = true;
                }
            }
            FetchingState::Push => {
                if self.fifo.len() != 0{
                    // wait until the fifo is empty, dont advance the state machine either
                    return;
                }
                // On DMG LCDC bit 0 turn the background to white but not on CGB
                if lcd_control & BIT_0_MASK == 0 && !cgb_enabled {
                    self.fifo.fill(&EMPTY_FIFO_BUFFER);
                    self.current_x_pos += SPRITE_WIDTH;
                }
                else{
                    let low_data = self.fetcher_state_machine.data.low_tile_data;
                    let high_data = self.fetcher_state_machine.data.high_tile_data;
                    let mut pixels = [
                        get_decoded_pixel(7, low_data, high_data),
                        get_decoded_pixel(6, low_data, high_data),
                        get_decoded_pixel(5, low_data, high_data),
                        get_decoded_pixel(4, low_data, high_data),
                        get_decoded_pixel(3, low_data, high_data),
                        get_decoded_pixel(2, low_data, high_data),
                        get_decoded_pixel(1, low_data, high_data),
                        get_decoded_pixel(0, low_data, high_data),
                    ];
                    if cgb_enabled && self.cgb_attribute.attribute.flip_x{
                        pixels.reverse();
                    }
                    for i in 0..SPRITE_WIDTH{
                        self.fifo.push(BackgroundPixel {color_index: pixels[i as usize], attributes: self.cgb_attribute});
                        self.current_x_pos += 1;

                        let last_rendering_status = self.rendering_window;
                        self.rendering_window = self.is_rendering_wnd(lcd_control, window_pos);            
                        // In case I was rendering a background pixel need to reset the state of the fetcher 
                        // (and maybe clear the fifo but right now Im not doing it since im not sure what about the current_x_pos var)
                        if self.rendering_window && !last_rendering_status{
                            self.fetcher_state_machine.reset();
                            return;
                        }
                    }
                }
            }
            FetchingState::Sleep => {}
        }
        self.fetcher_state_machine.advance();
    }

    fn get_tila_data_address(&self, lcd_control:u8, scy:u8, ly_register:u8, tile_num:u8, cgb_enabled:bool)->u16{
        let current_tile_base_data_address = if (lcd_control & BIT_4_MASK) == 0 && (tile_num & BIT_7_MASK) == 0 {0x1000} else {0};
        let current_tile_data_address = current_tile_base_data_address + (tile_num  as u16 * 16);
        let sprite_line_number = if self.rendering_window{
            (self.window_line_counter % SPRITE_WIDTH) as u16
        }else{
            (scy as u16 + ly_register as u16 ) % SPRITE_WIDTH as u16
        };
        // if flip_y is set I want to take the last line instead if first (and so on), so im substracting the line number in the sprite from 7(=maxlines - 1)
        return current_tile_data_address + 2 * 
            if cgb_enabled && self.cgb_attribute.attribute.flip_y{7 - sprite_line_number}else{sprite_line_number};
    }

    fn is_rendering_wnd(&self, lcd_control:u8, window_pos:&Vec2<u8>)->bool{
        window_pos.x <= self.current_x_pos && self.has_wy_reached_ly && (lcd_control & BIT_5_MASK) != 0
    }
}