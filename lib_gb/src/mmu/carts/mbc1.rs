use std::vec::Vec;
use super::mbc::*;


pub struct Mbc1{
    program:Vec<u8>,
    ram:Vec<u8>,
    register0:u8,
    register1:u8,
    register2:u8,
    register3:u8,
    battery:bool
}

impl Mbc for Mbc1{
    fn get_ram(&self) ->&[u8] {
        self.ram.as_slice()
    }

    fn has_battery(&self) ->bool {
        self.battery
    }

    fn read_bank0(&self, address: u16)->u8{
        self.program[address as usize]
    }

    fn read_current_bank(&self, address:u16)->u8{
        let bank:u16 = self.get_current_rom_bank() as u16;
        return self.program[ROM_BANK_SIZE as usize * bank as usize + address as usize];
    }

    fn write_rom(&mut self, address: u16, value: u8){
        match address{
            0..=0x1FFF      =>self.register0 = value,
            0x2000..=0x3FFF =>self.register1 = value,
            0x4000..=0x5FFF =>self.register2 = value,
            0x6000..=0x7FFF =>self.register3 = value,
            _=>std::panic!("cannot write to this address in bank0 in mbc1 cartridge")
        }
    }

    fn read_external_ram(&self, address: u16)->u8{
        if self.ram.is_empty(){
            return 0xFF;
        }
        let bank:u16 = self.get_current_ram_bank() as u16;
        return self.ram[(bank * RAM_BANK_SIZE + address) as usize];
    }

    fn write_external_ram(&mut self, address: u16, value: u8){
        if self.ram.len() > 0{
            let bank:u16 = self.get_current_ram_bank() as u16;
            self.ram[(bank * RAM_BANK_SIZE + address) as usize] = value;   
        }
    }
}

impl Mbc1{
    pub fn new(v:Vec<u8>, battery:bool, ram:Option<Vec<u8>>)->Self{
        let mut mbc = Mbc1{
            program:v,
            ram:Vec::new(),
            register0:0,
            register1:0,
            register2:0,
            register3:0,
            battery:battery
        };

        mbc.ram = init_ram(mbc.program[MBC_RAM_SIZE_LOCATION], ram);

        return mbc;
    }

    fn get_current_rom_bank(&self)->u8{
        let mut bank = self.register1 & 0b11111;

        //banks 0x0 0x20 0x40 0x60 are not avaalible through this method
        if bank == 0{
            bank+=1;
        }
        if self.register3 == 0{
            bank |= (self.register2 & 0b11)<<5;
        }

        return bank;
    }

    fn get_current_ram_bank(&self)->u8{
        if self.register3 == 1{
            return self.register2 & 0b11;
        }

        return 0;
    }
}