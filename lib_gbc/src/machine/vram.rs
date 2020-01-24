const VRAM_SIZE:usize = 0x4000;
const VRAM_BANK_SIZE:usize = 0x2000;
pub struct VRam{
    memory:[u8;VRAM_SIZE],
    current_bank_register:u8
}

impl VRam{
    pub fn read_bank0(&self, address:u16)->u8{
        return memory[address as usize];
    }

    pub fn read_current_bank(&self, address:u16)->u8{
        
    }
}