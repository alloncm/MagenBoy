use std::mem::{self, MaybeUninit};

use crate::{mmu::vram::VRam, ppu::sprite_attribute::SpriteAttribute};

use super::fetching_state::FethcingState;

pub struct SpriteFetcher{
    pub fifo:Vec<(u8, u8)>,
    pub oam_entries:[SpriteAttribute; 10],
    pub oam_entries_len:u8,

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
            fifo:Vec::<(u8,u8)>::with_capacity(8)
        }
    }

    pub fn reset(&mut self){
        self.current_oam_entry = 0;
        self.oam_entries_len = 0;
        self.current_fetching_state = FethcingState::TileNumber;
        self.fifo.clear();
    }

    pub fn fetch_pixels(&mut self, vram:&VRam, ly_register:u8, current_x_pos:u8){
        match self.current_fetching_state{
            FethcingState::TileNumber=>{
                if self.oam_entries_len > self.current_oam_entry{
                    let oam_entry = &self.oam_entries[self.current_oam_entry as usize];
                    if oam_entry.x <= current_x_pos + 8{
                        self.current_fetching_state = FethcingState::LowTileData(oam_entry.tile_number);
                    }
                }
            }
            FethcingState::LowTileData(tile_num)=>{
                let current_tile_data_address = tile_num as u16 * 16;
                let low_data = vram.read_current_bank(current_tile_data_address + (2 * (ly_register % 8)) as u16);
                self.current_fetching_state = FethcingState::HighTileData(tile_num, low_data);
            }
            FethcingState::HighTileData(tile_num, low_data)=>{
                let current_tile_data_address = tile_num as u16 * 16;
                let high_data = vram.read_current_bank(current_tile_data_address + (2 * (ly_register % 8)) as u16 + 1);
                self.current_fetching_state = FethcingState::Push(low_data, high_data);
            }
            FethcingState::Push(low_data, high_data)=>{
                let oam_attribute = &self.oam_entries[self.current_oam_entry as usize];

                if !oam_attribute.flip_x{
                    for i in (0..8).rev(){
                        let mask = 1 << i;
                        let mut pixel = (low_data & mask) >> i;
                        pixel |= ((high_data & mask) >> i) << 1;
                        self.fifo.push((pixel, self.current_oam_entry));
                    }
                }
                else{
                    for i in 0..8{
                        let mask = 1 << i;
                        let mut pixel = (low_data & mask) >> i;
                        pixel |= ((high_data & mask) >> i) << 1;
                        self.fifo.push((pixel, self.current_oam_entry));
                    }
                }

                self.current_fetching_state = FethcingState::TileNumber;

                self.current_oam_entry += 1;
            }
        }
    }
}