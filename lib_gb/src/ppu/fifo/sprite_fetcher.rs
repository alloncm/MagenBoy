use std::mem::{self, MaybeUninit};

use crate::{mmu::vram::VRam, ppu::sprite_attribute::SpriteAttribute, utils::bit_masks::{BIT_0_MASK, BIT_2_MASK}};

use super::fetching_state::FethcingState;

pub struct SpriteFetcher{
    pub fifo:Vec<(u8, u8)>,
    pub oam_entries:[SpriteAttribute; 10],
    pub oam_entries_len:u8,
    pub rendering:bool,

    current_fetching_state:FethcingState,
    current_oam_entry:u8,
}

impl SpriteFetcher{
    pub fn new()->Self{
        let oam_entries = {
            let mut data: [MaybeUninit<SpriteAttribute>; 10] = unsafe{
                MaybeUninit::uninit().assume_init()
            };

            for elem in &mut data[..]{
                *elem = MaybeUninit::new(SpriteAttribute::new(0, 0, 0, 0));
            }

            unsafe{mem::transmute::<_, [SpriteAttribute;10]>(data)}
        };
        
        SpriteFetcher{
            current_fetching_state:FethcingState::TileNumber,
            current_oam_entry:0,
            oam_entries_len:0,
            oam_entries,
            fifo:Vec::<(u8,u8)>::with_capacity(8),
            rendering:false
        }
    }

    pub fn reset(&mut self){
        self.current_oam_entry = 0;
        self.oam_entries_len = 0;
        self.current_fetching_state = FethcingState::TileNumber;
        self.fifo.clear();
        self.rendering = false;
    }

    pub fn fetch_pixels(&mut self, vram:&VRam, lcd_control:u8, ly_register:u8, current_x_pos:u8){
        let sprite_size = if lcd_control & BIT_2_MASK == 0 {8} else{16};

        match self.current_fetching_state{
            FethcingState::TileNumber=>{
                if self.oam_entries_len > self.current_oam_entry{
                    let oam_entry = &self.oam_entries[self.current_oam_entry as usize];
                    if oam_entry.x <= current_x_pos + 8 && current_x_pos < oam_entry.x{
                        let mut tile_number = oam_entry.tile_number;
                        if lcd_control & BIT_2_MASK != 0{
                            tile_number &= !BIT_0_MASK
                        }
                        self.current_fetching_state = FethcingState::LowTileData(tile_number);
                        self.rendering = true;
                        return;
                    }
                }
                // Reach here if not rendering this time a sprite
                self.rendering = false;
            }
            FethcingState::LowTileData(tile_num)=>{
                let oam_attribute = &self.oam_entries[self.current_oam_entry as usize];
                let current_tile_data_address = Self::get_current_tile_data_address(ly_register, oam_attribute, sprite_size, tile_num);
                let low_data = vram.read_current_bank(current_tile_data_address);
                self.current_fetching_state = FethcingState::HighTileData(tile_num, low_data);
            }
            FethcingState::HighTileData(tile_num, low_data)=>{
                let oam_attribute = &self.oam_entries[self.current_oam_entry as usize];
                let current_tile_data_address = Self::get_current_tile_data_address(ly_register, oam_attribute, sprite_size, tile_num);
                let high_data = vram.read_current_bank(current_tile_data_address + 1);
                self.current_fetching_state = FethcingState::Push(low_data, high_data);
            }
            FethcingState::Push(low_data, high_data)=>{
                let oam_attribute = &self.oam_entries[self.current_oam_entry as usize];
                let start_x = self.fifo.len();

                if oam_attribute.flip_x{
                    for i in 0..8{
                        let mask = 1 << i;
                        let mut pixel = (low_data & mask) >> i;
                        pixel |= ((high_data & mask) >> i) << 1;
                        if i < start_x {
                            if self.fifo[i].0 == 0{
                                // self.fifo[i].0 = pixel;
                            }
                        }
                        else{
                            self.fifo.push((pixel, self.current_oam_entry));
                        }
                    }
                }
                else{
                    for i in (0..8).rev(){
                        let mask = 1 << i;
                        let mut pixel = (low_data & mask) >> i;
                        pixel |= ((high_data & mask) >> i) << 1;
                        if 7 - i < start_x {
                            if self.fifo[7 - i].0 == 0{
                                // self.fifo[7 - i].0 = pixel;
                            }
                        }
                        else{
                            self.fifo.push((pixel, self.current_oam_entry));
                        }
                    }
                }

                self.current_fetching_state = FethcingState::TileNumber;
                self.current_oam_entry += 1;
            }
        }
    }

    // Receiving the tile_num since in case of extended sprite this could change (the first bit is reset)
    fn get_current_tile_data_address(ly_register:u8, sprite_attrib:&SpriteAttribute, sprite_size:u8, tile_num:u8)->u16{
        return if sprite_attrib.flip_y{
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