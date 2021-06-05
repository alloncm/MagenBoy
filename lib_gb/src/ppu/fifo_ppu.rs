use super::{ppu_state::PpuState, sprite_attribute::SpriteAttribute};

pub struct FifoPpu{
    oam_entries:[SpriteAttribute; 10],
    current_oam_entry:u8,
    t_cycles_passed:u16,
    state:PpuState,
    ly_register:u8,
}

impl FifoPpu{
    pub fn cycle(&mut self, m_cycles:u8, oam:&[u8;0xA0], extended_sprite:bool)->(){
        let sprite_height = if extended_sprite {16} else {8};

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

                }
            }
        }
    }
}

struct PixelFetcher{

}

impl PixelFetcher{
    pub fn cycle(&mut self){

    }
}