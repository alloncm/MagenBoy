pub mod memory;
pub mod gb_mmu;
pub mod ram;
pub mod vram;
pub mod io_ports;
pub mod carts;
pub mod access_bus;
pub mod io_bus;
pub mod interrupts_handler;
pub mod external_memory_bus;
pub mod oam_dma_controller;
pub mod vram_dma_controller;

pub use external_memory_bus::GB_BOOT_ROM_SIZE;
pub use external_memory_bus::GBC_BOOT_ROM_SIZE;