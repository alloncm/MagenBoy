
const RAM_SZIE:usize = 0x8000;
const BANK_SIZE:usize = 0x1000;

pub struct Ram{
    memory: [u8;RAM_SZIE],
    ram_bank_register:u8
}

impl Ram{
    pub fn read_bank0(&self, address:u16)->u8{
        return self.memory[address as usize];
    }

    pub fn read_current_bank(&self, address:u16)->u8{
        return self.memory[self.get_valid_address(address)];
    }

    pub fn write_bank0(&mut self, address:u16,value:u8){
        self.memory[address as usize] = value;
    }

    pub fn write_current_bank(&mut self, address:u16, value:u8){
        self.memory[self.get_valid_address(address)] = value;
    }

    pub fn set_bank(&mut self, mut bank:u8){
        if bank == 0{
            bank = 1;
        }
        
        self.ram_bank_register = bank & 0b111;
    }

    pub fn get_bank(&self)->u8{self.ram_bank_register}

    fn get_valid_address(&self, address:u16)->usize{
        return BANK_SIZE*(self.ram_bank_register as usize) + (address as usize);
    }
}

impl Default for Ram{
    fn default()->Ram{
        Ram{
            memory:[0;RAM_SZIE],
            ram_bank_register:1
        }
    }
}