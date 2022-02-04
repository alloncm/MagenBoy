use crate::ppu::{gb_ppu::GbPpu, gfx_device::GfxDevice};
use super::{oam_dma_transfer::OamDmaTransfer, external_memory_bus::ExternalMemoryBus, access_bus::AccessBus};

const DMA_SIZE:u16 = 0xA0;
const VRAM_BASE_ADDRESS:u16 = 0x8000;

pub struct OamDmaController{
    oam_dma_transfer:OamDmaTransfer
}

impl OamDmaController{
    pub fn new()->Self{
        Self{
            oam_dma_transfer: OamDmaTransfer::default()
        }
    }
    pub fn cycle<G:GfxDevice>(&mut self, m_cycles:u32, external_bus: &mut ExternalMemoryBus, ppu:&mut GbPpu<G>)->Option<AccessBus>{
        if let Some(bus) = self.oam_dma_transfer.enable{
            let cycles_to_run = std::cmp::min(self.oam_dma_transfer.dma_cycle_counter + m_cycles as u16, DMA_SIZE);
            match bus{
                AccessBus::External=>{
                    for i in self.oam_dma_transfer.dma_cycle_counter..cycles_to_run as u16{
                        let source_value = external_bus.read(self.oam_dma_transfer.soure_address + i);
                        ppu.oam[i as usize] = source_value;
                    }
                }
                AccessBus::Video=>{
                    let base_source_address = self.oam_dma_transfer.soure_address - VRAM_BASE_ADDRESS;
                    for i in self.oam_dma_transfer.dma_cycle_counter..cycles_to_run as u16{
                        let source_value = ppu.vram.read_current_bank(base_source_address + i);
                        ppu.oam[i as usize] = source_value;
                    }
                }
            }

            self.oam_dma_transfer.dma_cycle_counter += m_cycles as u16;
            if self.oam_dma_transfer.dma_cycle_counter >= DMA_SIZE{
                self.oam_dma_transfer.dma_cycle_counter = 0;
                self.oam_dma_transfer.enable = Option::None;
            }
        }

        return self.oam_dma_transfer.enable;
    }

    pub fn get_dma_register(&self)->u8{
        (self.oam_dma_transfer.soure_address >> 8) as u8
    }

    pub fn set_dma_register(&mut self, value:u8){
        let address = (value as u16) << 8;
        self.oam_dma_transfer.soure_address = address;
        self.oam_dma_transfer.enable = match value{
            0..=0x7F=>Some(AccessBus::External),
            0x80..=0x9F=>Some(AccessBus::Video),
            0xA0..=0xFF=>Some(AccessBus::External)
        }
    }
}