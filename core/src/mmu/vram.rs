const VRAM_SIZE:usize = 0x4000;
const VRAM_BANK_SIZE:usize = 0x2000;

pub struct VRam{
    memory:[u8;VRAM_SIZE],
    current_bank_register:u8
}

impl VRam{
    pub fn set_bank(&mut self, bank:u8){
        self.current_bank_register = bank & 0b1;
    }

    pub fn get_bank(&self)->u8{self.current_bank_register}

    pub fn read_current_bank(&self, address:u16)->u8{
        return self.memory[self.get_valid_address(address)];
    }

    pub fn read_bank(&self, address:u16, bank:u8)->u8{self.memory[(VRAM_BANK_SIZE * bank as usize) + address as usize]}

    pub fn write_current_bank(&mut self, address:u16, value:u8){
        self.memory[self.get_valid_address(address)] = value;
    }

    fn get_valid_address(&self, address:u16)->usize{
        return (address as usize) + ((self.current_bank_register as usize)*VRAM_BANK_SIZE);
    }
}

impl Default for VRam{
    fn default()->VRam{
        VRam{
            memory:[0;VRAM_SIZE],
            current_bank_register:0
        }
    }
}

impl Clone for VRam{
    fn clone(&self)->VRam{
        VRam{
            memory:self.memory,
            current_bank_register:self.current_bank_register
        }
    }
}