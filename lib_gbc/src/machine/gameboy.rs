use crate::cpu::gbc_cpu::GbcCpu;
use crate::machine::memory::Memory;

const SCREEN_HEIGHT: u8 = 144;
const SCREEN_WIDTH: u8 = 160;

pub struct GameBoy {
    pub cpu: GbcCpu,
    pub mmu:dyn Memory
}
