use crate::{mmu::vram::VRam, ppu::attributes::SpriteAttributes, utils::{self, bit_masks::{BIT_0_MASK, BIT_2_MASK}, fixed_size_queue::FixedSizeQueue}, machine::Mode};
use super::{FIFO_SIZE, SPRITE_WIDTH, fetching_state::*, get_decoded_pixel};

pub const NORMAL_SPRITE_HIGHT:u8 = 8;
pub const EXTENDED_SPRITE_HIGHT:u8 = 16;
pub const MAX_SPRITES_PER_LINE:usize = 10;

#[derive(Clone, Copy, Default)]
pub struct SpritePixel {pub color_index:u8, pub oam_entry:u8}

pub struct SpriteFetcher{
    pub fifo:FixedSizeQueue<SpritePixel, FIFO_SIZE>,
    pub oam_entries:[SpriteAttributes; 10],
    pub oam_entries_len:u8,
    pub rendering:bool,

    fetcher_state_machine:FetcherStateMachine,
    current_oam_entry:u8,
    mode:Mode
}

impl SpriteFetcher{
    pub fn new(mode:Mode)->Self{
        let oam_entries:[SpriteAttributes; MAX_SPRITES_PER_LINE] = utils::create_array(|| SpriteAttributes::new(0,0,0,0, 0, 0));
        let state_machine:[FetchingState;8] = [FetchingState::FetchTileNumber, FetchingState::Sleep, FetchingState::Sleep, FetchingState::FetchLowTile, FetchingState::Sleep, FetchingState::FetchHighTile, FetchingState::Sleep, FetchingState::Push];
        
        SpriteFetcher{
            fetcher_state_machine:FetcherStateMachine::new(state_machine),
            current_oam_entry:0,
            oam_entries_len:0,
            oam_entries,
            fifo:FixedSizeQueue::<SpritePixel, 8>::new(),
            rendering:false,
            mode
        }
    }

    pub fn reset(&mut self){
        self.current_oam_entry = 0;
        self.oam_entries_len = 0;
        self.fetcher_state_machine.reset();
        self.fifo.clear();
        self.rendering = false;
    }

    pub fn fetch_pixels(&mut self, vram:&VRam, lcd_control:u8, ly_register:u8, current_x_pos:u8){
        let sprite_size = if lcd_control & BIT_2_MASK == 0 {NORMAL_SPRITE_HIGHT} else{EXTENDED_SPRITE_HIGHT};

        match self.fetcher_state_machine.current_state(){
            FetchingState::FetchTileNumber=>{
                self.try_fetch_tile_number(current_x_pos, lcd_control, sprite_size, ly_register);
            }
            FetchingState::FetchLowTile=>{
                let oam_attribute = &self.oam_entries[self.current_oam_entry as usize];
                let bank = if self.mode == Mode::CGB {oam_attribute.attributes.gbc_bank as u8}else{0};
                let low_data = vram.read_bank(self.fetcher_state_machine.data.tile_data_address, bank);
                self.fetcher_state_machine.data.low_tile_data = low_data;
                self.fetcher_state_machine.advance();
            }
            FetchingState::FetchHighTile=>{
                let oam_attribute = &self.oam_entries[self.current_oam_entry as usize];
                let bank = if self.mode == Mode::CGB {oam_attribute.attributes.gbc_bank as u8}else{0};
                let high_data = vram.read_bank(self.fetcher_state_machine.data.tile_data_address + 1, bank);
                self.fetcher_state_machine.data.high_tile_data = high_data;
                self.fetcher_state_machine.advance();
            }
            FetchingState::Push=>{
                let low_data = self.fetcher_state_machine.data.low_tile_data;
                let high_data = self.fetcher_state_machine.data.high_tile_data;
                let oam_attribute = &self.oam_entries[self.current_oam_entry as usize];
                let start_x = oam_attribute.visibility_start as usize;
                let end_x = oam_attribute.visibility_end as usize;
                let mut pixels = [
                    get_decoded_pixel(0, low_data, high_data),
                    get_decoded_pixel(1, low_data, high_data),
                    get_decoded_pixel(2, low_data, high_data),
                    get_decoded_pixel(3, low_data, high_data),
                    get_decoded_pixel(4, low_data, high_data),
                    get_decoded_pixel(5, low_data, high_data),
                    get_decoded_pixel(6, low_data, high_data),
                    get_decoded_pixel(7, low_data, high_data),
                ];

                if !oam_attribute.attributes.flip_x{
                    pixels.reverse();
                }

                if self.fifo.len() >= start_x{
                    for i in 0..start_x{
                        // pixel 0 is transparent
                        if self.fifo[i].color_index == 0{
                            self.fifo[i] = SpritePixel{color_index: pixels[i], oam_entry:self.current_oam_entry};
                        }
                    }
                }
                for i in start_x..end_x{
                    if self.fifo.len() <= i{
                        self.fifo.push(SpritePixel{color_index:pixels[i], oam_entry:self.current_oam_entry});
                    }
                    else if pixels[i] != 0{
                        self.fifo[i] = SpritePixel{color_index:pixels[i], oam_entry:self.current_oam_entry};
                    }
                }

                // if end_x is greater or equal than start_x it means that those pixels didnt had a chance to be pushed to the fifo
                if self.mode == Mode::CGB && end_x >= start_x{
                    // support CGB mode with sprite with lower priority (high OAM index) at a small x
                    // and a sprite with higher prority (low OAM index) at high x
                    let start_index =  core::cmp::max(end_x, self.fifo.len());
                    for i in start_index..FIFO_SIZE{
                        self.fifo.push(SpritePixel { color_index: pixels[i], oam_entry: self.current_oam_entry});
                    }
                }

                self.current_oam_entry += 1;
                self.fetcher_state_machine.advance();
            }
            FetchingState::Sleep=>self.fetcher_state_machine.advance()
        }
    }

    //This is a function on order to abort if rendering
    fn try_fetch_tile_number(&mut self, current_x_pos: u8, lcd_control: u8, sprite_size:u8, ly_register:u8) {
        if self.oam_entries_len > self.current_oam_entry{
            let oam_entry = &self.oam_entries[self.current_oam_entry as usize];
            if oam_entry.x <= current_x_pos + SPRITE_WIDTH && current_x_pos < oam_entry.x{
                let mut tile_number = oam_entry.tile_number;
                if lcd_control & BIT_2_MASK != 0{
                    tile_number &= !BIT_0_MASK
                }
                self.rendering = true;
                self.fetcher_state_machine.data.reset();
                self.fetcher_state_machine.data.tile_data = tile_number;
                self.fetcher_state_machine.data.tile_data_address = Self::get_current_tile_data_address(ly_register, 
                    &self.oam_entries[self.current_oam_entry as usize], sprite_size, tile_number);
                self.fetcher_state_machine.advance();
                return;
            }
        }
        self.rendering = false;
    }

    // Receiving the tile_num since in case of extended sprite this could change (the first bit is reset)
    fn get_current_tile_data_address(ly_register:u8, sprite_attrib:&SpriteAttributes, sprite_size:u8, tile_num:u8)->u16{
        return if sprite_attrib.attributes.flip_y{
            // Since Im flipping but dont know for what rect (8X8 or 8X16) I need sub this from the size (minus 1 casue im starting to count from 0 in the screen lines).
            tile_num as u16 * 16 + (2 * (sprite_size - 1 - (16 - (sprite_attrib.y - ly_register)))) as u16
        }
        else{
            // Since the sprite attribute y pos is the right most dot of the rect 
            // Im subtracting this from 16 (since the rects are 8X16)
            tile_num as u16 * 16 + (2 * (16 - (sprite_attrib.y - ly_register))) as u16
        };
    }
}