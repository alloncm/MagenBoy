use crate::{utils::bit_masks::BIT_7_MASK, ppu::{gb_ppu::GbPpu, gfx_device::GfxDevice, ppu_state::PpuState}};

use super::external_memory_bus::ExternalMemoryBus;

enum TransferMode{
    GeneralPurpose,
    Hblank,
    Terminated
}

const HBLANK_TRANSFER_CHUNK_SIZE:u8 = 0x10;
const BYTES_TRASNFERED_PER_M_CYCLE:u8 = 2;

pub struct VramDmaController{
    source_address:u16,
    dest_address:u16,
    mode:TransferMode,
    remaining_length:u8,

    last_ly:Option<u8>,
    hblank_transfer_burst_counter:u8
}

impl VramDmaController{
    pub fn new()->Self{
        Self{
            dest_address: 0,
            last_ly: None,
            hblank_transfer_burst_counter: 0,
            mode: TransferMode::Terminated,
            remaining_length: 0,
            source_address: 0
        }
    }

    pub fn set_source_high(&mut self, value:u8){
        self.source_address = (self.source_address & 0x00FF) | (value as u16) << 8;
    }
    
    pub fn set_source_low(&mut self, value:u8){
        // Ignores the last 4 bits of the source
        let value = value & 0xF0;
        self.source_address = (self.source_address & 0xFF00) | value as u16;
    }

    pub fn set_dest_high(&mut self, value:u8){
        // Upper 3 bits are ignored since the dest are always in the vram
        let value = value & 0b0001_1111;
        self.dest_address = (self.dest_address & 0x00FF) | (value as u16) << 8;
    }
    
    pub fn set_dest_low(&mut self, value:u8){
        // Ignores the last 4 bits of the dest
        let value = value & 0xF0;
        self.dest_address = (self.dest_address & 0xFF00) | value as u16;
    }

    pub fn set_mode_length(&mut self, value:u8){
        match self.mode{
            TransferMode::Hblank=> if value & BIT_7_MASK == 0 {
                self.mode = TransferMode::Terminated;
                self.last_ly = None;
            },
            TransferMode::Terminated=>{
                self.mode = 
                    if (value & BIT_7_MASK) == 0{
                        log::info!("Set DMA GP");
                        TransferMode::GeneralPurpose}
                    else{
                        log::info!("Set DMA HBlank");
                        TransferMode::Hblank
                    };
                self.remaining_length = (value & !BIT_7_MASK) + 1;
            }
            TransferMode::GeneralPurpose=>core::panic!("Cant pause DMA GP transfer")
        }
    }

    pub fn get_mode_length(&self)->u8{
        self.remaining_length.wrapping_sub(1)
    }

    pub fn should_block_cpu(&self)->bool{
        return match self.mode {
            TransferMode::Terminated |
            TransferMode::Hblank => false,
            TransferMode::GeneralPurpose => true
        };
    }

    pub fn cycle<G:GfxDevice>(&mut self, m_cycles:u32, exteranl_memory_bus:&mut ExternalMemoryBus, ppu:&mut GbPpu<G>){
        match self.mode{
            TransferMode::Hblank=>self.handle_hblank_transfer(ppu, m_cycles, exteranl_memory_bus),
            TransferMode::GeneralPurpose=>self.handle_general_purpose_transfer(exteranl_memory_bus, ppu, m_cycles),
            TransferMode::Terminated=>{}
        }
    }

    fn handle_general_purpose_transfer<G:GfxDevice>(&mut self, exteranl_memory_bus: &mut ExternalMemoryBus, ppu: &mut GbPpu<G>, m_cycles:u32) {
        for _ in 0..m_cycles {
            for _ in 0..BYTES_TRASNFERED_PER_M_CYCLE{
                let source_value = exteranl_memory_bus.read(self.source_address);
                ppu.vram.write_current_bank(self.dest_address, source_value);

                self.source_address += 1;
                self.dest_address += 1;
            }

            if self.source_address & 0xF == 0{
                // if fisnished 0x10 bytes transfer decrease length remaining
                self.remaining_length -= 1;
                if self.remaining_length == 0{
                    // end of dma transfer
                    self.mode = TransferMode::Terminated;
                    break;
                }
            }
        }
    }

    fn handle_hblank_transfer<G:GfxDevice>(&mut self, ppu: &mut GbPpu<G>, m_cycles: u32, exteranl_memory_bus: &mut ExternalMemoryBus) {
        if self.last_ly.is_some_and(|v|v == ppu.ly_register) || ppu.state != PpuState::Hblank {
            return;
        }

        for _ in 0..(m_cycles) {
            for _ in 0..BYTES_TRASNFERED_PER_M_CYCLE{
                let source_value = exteranl_memory_bus.read(self.source_address);
                ppu.vram.write_current_bank(self.dest_address, source_value);
                log::info!("{:#X} -> {:#X}, {:#X}", self.source_address, self.dest_address, source_value);
                self.source_address += 1;
                self.dest_address += 1;
            }

            self.hblank_transfer_burst_counter += BYTES_TRASNFERED_PER_M_CYCLE;

            if self.hblank_transfer_burst_counter == HBLANK_TRANSFER_CHUNK_SIZE {
                self.hblank_transfer_burst_counter = 0;
                self.last_ly = Some(ppu.ly_register);
                self.remaining_length -= 1;
                if self.remaining_length == 0{
                    self.mode = TransferMode::Terminated;
                    self.last_ly = None;
                }
                return;
            }
        }
    }
}