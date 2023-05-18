use crate::ppu::{gb_ppu::GbPpu, gfx_device::GfxDevice};
use super::{external_memory_bus::ExternalMemoryBus, access_bus::AccessBus};

const DMA_SIZE:u16 = 0xA0;
const VRAM_BASE_ADDRESS:u16 = 0x8000;

pub struct OamDmaController{
    soure_address:u16,
    enable:Option<AccessBus>,
    dma_cycle_counter:u16
}

impl OamDmaController{
    pub fn new()->Self{
        Self{dma_cycle_counter:0, enable:None, soure_address:0}
    }
    pub fn cycle<G:GfxDevice>(&mut self, m_cycles:u32, external_bus: &mut ExternalMemoryBus, ppu:&mut GbPpu<G>)->Option<AccessBus>{
        if let Some(bus) = self.enable{
            let cycles_to_run = core::cmp::min(self.dma_cycle_counter + m_cycles as u16, DMA_SIZE);
            match bus{
                AccessBus::External=>{
                    for i in self.dma_cycle_counter..cycles_to_run as u16{
                        let source_value = external_bus.read(self.soure_address + i);
                        ppu.oam[i as usize] = source_value;
                    }
                }
                AccessBus::Video=>{
                    let base_source_address = self.soure_address - VRAM_BASE_ADDRESS;
                    for i in self.dma_cycle_counter..cycles_to_run as u16{
                        let source_value = ppu.vram.read_current_bank(base_source_address + i);
                        ppu.oam[i as usize] = source_value;
                    }
                }
            }

            self.dma_cycle_counter += m_cycles as u16;
            if self.dma_cycle_counter >= DMA_SIZE{
                self.dma_cycle_counter = 0;
                self.enable = Option::None;
            }
        }

        return self.enable;
    }

    pub fn get_dma_register(&self)->u8{
        (self.soure_address >> 8) as u8
    }

    pub fn set_dma_register(&mut self, value:u8){
        let address = (value as u16) << 8;
        self.soure_address = address;
        self.enable = match value{
            0..=0x7F | 0xA0..=0xFF=> Some(AccessBus::External),
            0x80..=0x9F=> Some(AccessBus::Video),
        }
    }
}