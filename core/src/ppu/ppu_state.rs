#[repr(u8)]
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

impl PartialEq for PpuState{
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl PpuState{
    pub fn from_u8(mut value:u8)->Self{
        value = value & 0b0000_0011;
        return unsafe{core::mem::transmute::<u8, Self>(value)};
    }
}
