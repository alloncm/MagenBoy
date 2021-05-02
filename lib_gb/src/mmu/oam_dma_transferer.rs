use super::{access_bus::AccessBus, memory::UnprotectedMemory};

const DMA_SIZE: u16 = 0xA0;
const DMA_DEST: u16 = 0xFE00;

pub struct OamDmaTransferer {
    pub soure_address: u16,
    pub enable: Option<AccessBus>,
    dma_cycle_counter: u16,
}

impl Default for OamDmaTransferer {
    fn default() -> Self {
        OamDmaTransferer {
            dma_cycle_counter: 0,
            enable: None,
            soure_address: 0,
        }
    }
}

impl OamDmaTransferer {
    pub fn cycle(&mut self, memory: &mut impl UnprotectedMemory, m_cycles: u8) {
        if self.enable.is_some() {
            let cycles_to_run = std::cmp::min(self.dma_cycle_counter + m_cycles as u16, DMA_SIZE);
            for i in self.dma_cycle_counter..cycles_to_run as u16 {
                memory.write_unprotected(
                    DMA_DEST + i,
                    memory.read_unprotected(self.soure_address + i),
                );
            }

            self.dma_cycle_counter += m_cycles as u16;

            if self.dma_cycle_counter >= DMA_SIZE {
                self.enable = None;
                self.dma_cycle_counter = 0;
            }
        }
    }
}
