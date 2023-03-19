use super::*;

const ENABLE_RAM_VALUE:u8 = 0xA;

pub struct Mbc5{
    program:Vec<u8>,
    ram:Vec<u8>,
    battery:bool,
    ram_enable_register:u8,
    rom_bank_number_register:u16,
    ram_bank_number:u8,
}

impl Mbc for Mbc5 {
    fn get_ram(&self)->&[u8] {
        &self.ram
    }

    fn has_battery(&self)->bool {
        self.battery
    }

    fn read_bank0(&self, address:u16)->u8 {
        self.program[address as usize]
    }

    fn read_current_bank(&self, address:u16)->u8 {
        // bank number between 0..0x1FF (9bits)
        let bank = (self.rom_bank_number_register & 0x1FF) as usize * ROM_BANK_SIZE;
        self.program[address as usize + bank]
    }

    fn write_rom(&mut self, address:u16, value:u8) {
        let last_address_nible = (address >> 12) as u8;
        match last_address_nible{
            0|1=>self.ram_enable_register = value,
            // low 8 bits
            2=>self.rom_bank_number_register = (self.rom_bank_number_register & 0xFF00) | value as u16,
            // high bit 9
            3=>self.rom_bank_number_register = (self.rom_bank_number_register & 0x00FF) | ((value as u16) << 8),
            4|5=>self.ram_bank_number = value,
            _=>{}
        }
    }

    fn read_external_ram(&self, address:u16)->u8 {
        if self.ram_enable_register == ENABLE_RAM_VALUE{
            let bank = (self.ram_bank_number & 0xF) as usize * RAM_BANK_SIZE;
            return self.ram[address as usize + bank];
        }

        // ram is disabled
        return 0xFF;
    }

    fn write_external_ram(&mut self, address:u16, value:u8) {
        if self.ram_enable_register == ENABLE_RAM_VALUE{
            let bank = (self.ram_bank_number & 0xF) as usize * RAM_BANK_SIZE;
            self.ram[address as usize + bank] = value;
        }
        else{
            log::warn!("MBC5 write while ram is not enabled. ram_address: {}, value: {}", address, value);
        }
    }
}

impl Mbc5{
    pub fn new(program:Vec<u8>, battery:bool, ram:Option<Vec<u8>>)->Self{
        let mut mbc = Self{
            program,
            ram: Vec::new(),
            battery,
            ram_enable_register: 0,
            rom_bank_number_register: 0,
            ram_bank_number: 0,
        };

        mbc.ram = init_ram(mbc.program[MBC_RAM_SIZE_LOCATION], ram);

        return mbc;
    }
}