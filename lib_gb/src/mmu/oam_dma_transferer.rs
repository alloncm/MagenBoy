use super::access_bus::AccessBus;

pub struct OamDmaTransferer{
    pub soure_address:u16,
    pub enable:Option<AccessBus>,
    pub dma_cycle_counter:u16
}

impl Default for OamDmaTransferer{
    fn default() -> Self {
        OamDmaTransferer{dma_cycle_counter:0, enable:None, soure_address:0}
    }
}