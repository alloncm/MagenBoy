pub mod gb_mmu;
pub mod ram;
pub mod io_ports;
pub mod carts;
pub mod access_bus;
pub mod io_bus;
pub mod interrupts_handler;
pub mod external_memory_bus;
pub mod oam_dma_controller;
pub mod vram_dma_controller;

pub use external_memory_bus::{GB_BOOT_ROM_SIZE, GBC_BOOT_ROM_SIZE};

pub trait Memory{
    fn read(&mut self, address:u16, m_cycles:u8)->u8;
    fn write(&mut self, address:u16, value:u8, m_cycles:u8);
    fn set_double_speed_mode(&mut self, state:bool);
    fn set_halt(&mut self, state:bool);
}