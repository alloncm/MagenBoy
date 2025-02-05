use super::*;

pub struct Rom<'a>{
    program: &'a[u8],
    external_ram:&'static mut[u8],
    battery:bool
}

impl<'a> Mbc for Rom<'a>{
    
    fn get_ram(&mut self) ->&mut [u8] {
        self.external_ram
    }

    fn has_battery(&self) ->bool {
        self.battery
    }

    fn read_bank0(&self, address:u16)->u8{
        return self.program[address as usize];
    }

    fn write_rom(&mut self, _address: u16, _value: u8){
        //Just ignoring this as some games accidently wrtite here (ahhm Tetris)
    }

    fn read_current_bank(&self, address:u16)->u8{
        return self.program[ROM_BANK_SIZE + (address as usize)];
    }

    fn read_external_ram(&self, address:u16)->u8{
        self.external_ram[get_external_ram_valid_address(address as usize, &self.external_ram)]
    }

    fn write_external_ram(&mut self, address:u16, value:u8){
        self.external_ram[get_external_ram_valid_address(address as usize, &self.external_ram)] = value
    }

    #[cfg(feature = "dbg")]
    fn get_bank_number(&self)->u16 { 1 }
}

impl<'a> Rom<'a>{
    
    pub fn new(program:&'a[u8], battery:bool, ram:Option<&'static mut [u8]>)->Self{
        let ram_reg = program[MBC_RAM_SIZE_LOCATION];
        let external_ram = init_ram(ram_reg, ram);
        return Self{
            program,
            external_ram,
            battery
        };
    }
}