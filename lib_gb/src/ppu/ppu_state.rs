pub enum PpuState{
    Hblank = 0b00,
    Vblank = 0b01,
    OamSearch = 0b10,
    PixelTransfer = 0b11
}

impl Copy for PpuState{}

impl Clone for PpuState{
    fn clone(&self)->Self{
        *self
    }
}
