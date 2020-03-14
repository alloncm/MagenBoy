use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;
use crate::machine::gbc_memory::GbcMmu;
use crate::ppu::gbc_ppu::GbcPpu;
use crate::opcodes::opcodes_utils::*;


pub struct RegisterHandler<'a>{
    cpu:&'a mut GbcCpu,
    memory:&'a mut GbcMmu,
    ppu:&'a mut GbcPpu<'a>,                                                                          
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

        //updates ly register
        if register & BIT_7_MASK == 0{
            self.memory.write(0xFF44,0);
        }
    }

    fn handle_lcdstatus_register(&mut self, register:u8){
        let mut coincidence:u8 = (self.memory.read(0xFF44) == self.memory.read(0xFF45)) as u8;
        //to match the 2 bit
        coincidence <<=2;
        self.memory.write(0xFF41, register | coincidence);
    }

    fn handle_scroll_registers(&mut self,scroll_x:u8, scroll_y:u8){
        self.ppu.background_scroll.x = scroll_x;
        self.ppu.background_scroll.y = scroll_y;
    }

    fn handle_vrambank_register(&mut self, register:u8){
        if self.cpu.cgb_mode{
            self.memory.vram.set_bank(register & BIT_0_MASK);
        }
    }

    fn handle_switch_mode_register(&mut self, register:u8){
        if register & BIT_0_MASK != 0{
            self.cpu.cgb_mode = !self.cpu.cgb_mode;
            let cgb_mask = (self.cpu.cgb_mode as u8) <<7;
            self.memory.write(0xFF4D, register | cgb_mask);
        }
    }

    fn handle_wrambank_register(&mut self, register:u8){
        let bank:u8 = register & 0b00000111;
        self.memory.ram.set_bank(bank);
    }
}