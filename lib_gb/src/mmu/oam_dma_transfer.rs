use super::access_bus::AccessBus;

pub struct OamDmaTransfer{
    pub soure_address:u16,
    pub enable:Option<AccessBus>,
    pub dma_cycle_counter:u16
}

impl Default for OamDmaTransfer{
    fn default() -> Self {
        OamDmaTransfer{dma_cycle_counter:0, enable:None, soure_address:0}
    }
}