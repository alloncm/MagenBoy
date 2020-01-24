use crate::cpu::gbc_cpu::GbcCpu;

const MEMORY_SIZE: usize = 0x8000;
const VIDEO_MEMORY_SIZE: usize = 0x4000;
const SCREEN_HEIGHT: u8 = 144;
const SCREEN_WIDTH: u8 = 160;

pub struct GameBoy {
    pub cpu: GbcCpu,
    pub memory: [u8; MEMORY_SIZE],
    pub video_memory: [u8; VIDEO_MEMORY_SIZE],
}
