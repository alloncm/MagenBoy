use crate::{ppu::ppu_state::PpuState, utils::memory_registers::*};
use super::{access_bus::AccessBus, gb_mmu::GbMmu, io_ports::IO_PORTS_MEMORY_OFFSET, memory::UnprotectedMemory, oam_dma_transferer::OamDmaTransferer};

const DMA_REGISTER_INDEX:usize = (DMA_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;

pub fn update_mmu_registers(memory: &mut GbMmu,dma:&mut OamDmaTransferer){
     
    handle_ppu_state(memory, memory.read_unprotected(STAT_REGISTER_ADDRESS));
    handle_wram_register(memory, memory.read_unprotected(SVBK_REGISTER_ADDRESS));
    handle_bootrom_register(memory, memory.read_unprotected(BOOT_REGISTER_ADDRESS));
    let ports = memory.io_ports.get_ports_cycle_trigger();
    if ports[DMA_REGISTER_INDEX]{
        ports[DMA_REGISTER_INDEX] = false;
        handle_dma_transfer_register(memory.read_unprotected(DMA_REGISTER_ADDRESS), dma, memory);
    }
    else{
        memory.dma_state = dma.enable;
    }
}

fn handle_ppu_state(memory:&mut GbMmu, stat:u8){
    memory.ppu_state = PpuState::from_u8(stat & 0b0000_0011);
}

fn handle_wram_register(memory: &mut GbMmu, register:u8){
    let bank:u8 = register & 0b00000111;
    memory.ram.set_bank(bank);
}

fn handle_bootrom_register(memory: &mut GbMmu, register:u8){
    memory.finished_boot = register == 1;
}

fn handle_dma_transfer_register(register:u8, dma: &mut OamDmaTransferer, mmu:&mut GbMmu){
    dma.soure_address = (register as u16) << 8;
    dma.enable = match register{
        0..=0x7F=>Some(AccessBus::External),
        0x80..=0x9F=>Some(AccessBus::Video),
        0xA0..=0xFF=>Some(AccessBus::External)
    };

    mmu.dma_state = dma.enable;
}