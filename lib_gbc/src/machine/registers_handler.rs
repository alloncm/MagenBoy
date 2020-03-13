use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;
use crate::ppu::gbc_ppu::GbcPpu;


pub struct RegisterHandler<'a>{
    cpu:&'a mut GbcCpu,
    memory:&'a dyn Memory,
    ppu:&'a mut GbcPpu
}

impl<'a> RegisterHandler<'a>{
    pub fn update_state(){

    }

    fn update_
}