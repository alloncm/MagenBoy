
const RAM_SZIE:usize = 0x8000;
const RAM_POS_BANK0:u16 = 0xC000;
const RAM_POS_BANKS:u16 = 0xD000;
const BANK_SIZE:u16 = 0x1000;

pub struct Ram{
    memory: [u8;RAM_SZIE],
    ram_bank_register:u8
}

impl Ram{
    pub fn read_bank0(&self, address:u16)->u8{
        return self.memory[address as usize];
    }

    pub fn read_current_bank(&self, address:u16)->u8{
        let ram_address = BANK_SIZE*(self.ram_bank_register as u16);
        return self.memory[ram_address as usize];
    }

    pub fn set_bank(&mut self, bank:u8){
        if bank == 0{
            bank = 1;
        }
        self.ram_bank_register = bank;
    }
}