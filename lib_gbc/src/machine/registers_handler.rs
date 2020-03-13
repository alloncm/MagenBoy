use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;
use crate::ppu::gbc_ppu::GbcPpu;
use crate::opcodes::opcodes_utils::*;


pub struct RegisterHandler<'a>{
    cpu:&'a mut GbcCpu,
    memory:&'a dyn Memory,
    ppu:&'a mut GbcPpu<'a>
}

impl<'a> RegisterHandler<'a>{
    pub fn update_state(){

    }

    fn handle_lcdcontrol_register(&mut self, register:u8){
        self.ppu.screen_enable = (register & BIT_7_MASK) != 0;
        self.ppu.window_tile_map_address = (register & BIT_6_MASK) != 0;
        self.ppu.window_enable = (register & BIT_5_MASK) != 0;
        self.ppu.window_tile_background_map_data_address = (register & BIT_4_MASK) != 0;
        self.ppu.background_tile_map_address = (register & BIT_3_MASK) != 0;
        self.ppu.sprite_extended = (register & BIT_2_MASK) != 0;
        self.ppu.sprite_enable = (register & BIT_1_MASK) != 0;
        self.ppu.background_enabled = (register & BIT_0_MASK) != 0;
    }
}