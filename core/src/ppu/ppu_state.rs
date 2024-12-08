#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum PpuState{
    Hblank = 0b00,
    Vblank = 0b01,
    OamSearch = 0b10,
    PixelTransfer = 0b11
}