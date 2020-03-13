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
        self.ppu.windows_enable = (register & BIT_5_MASK) != 0;
        self.ppu.background_enabled = (register & BIT_0_MASK) != 0;
    }

}