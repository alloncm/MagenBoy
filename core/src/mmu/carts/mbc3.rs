use crate::utils::bit_masks::BIT_7_MASK;

use super::*;

const RAM_TIMER_ENABLE_VALUE:u8 = 0xA;
const EXTERNAL_RAM_READ_ERROR_VALUE:u8 = 0xFF;
const RTC_REGISTERS_COUNT:usize = 5;

pub struct Mbc3<'a>{
    program:&'a[u8],
    ram:&'a mut[u8],
    battery:bool,
    current_bank:u8, 
    ram_timer_enable:u8,
    ram_rtc_select:u8,
    latch_clock_data:u8,
    rtc_registers:[u8;RTC_REGISTERS_COUNT]
}

impl<'a> Mbc for Mbc3<'a>{

    fn get_ram(&mut self)->&mut[u8] {
        self.ram
    }

    fn has_battery(&self)->bool {
        self.battery
    }

    fn read_bank0(&self, address:u16)->u8{
        self.program[address as usize]
    }

    fn read_current_bank(&self, address: u16)->u8{
        let current_bank = self.get_current_rom_bank() as u16;
        let internal_address:usize = (ROM_BANK_SIZE as usize* current_bank as usize) + address as usize;

        self.program[internal_address]
    }

    fn write_rom(&mut self, address: u16, value: u8){
        match address{
            0..=0x1FFF=>self.ram_timer_enable = value,
            0x2000..=0x3FFF=>self.current_bank = value,
            0x4000..=0x5FFF=>self.ram_rtc_select = value,
            0x6000..=0x7FFF=>self.latch_clock_data = value,
            _=>core::panic!("cannot write to this address in mbc3 cartridge")
        }
    }

    fn read_external_ram(&self, address: u16)->u8{
        if self.ram_timer_enable != RAM_TIMER_ENABLE_VALUE{
            return EXTERNAL_RAM_READ_ERROR_VALUE;
        }
        
        return match self.ram_rtc_select{
            0..=3=>{
                let internal_address = self.ram_rtc_select as usize * RAM_BANK_SIZE as usize +  address as usize;
                let address = get_external_ram_valid_address(internal_address, &self.ram);
                return self.ram[address];
            },
            0x8..=0xC=>self.rtc_registers[(self.ram_rtc_select - 8) as usize],
            _=>EXTERNAL_RAM_READ_ERROR_VALUE
        };
    }

    fn write_external_ram(&mut self, address: u16, value: u8){
        if self.ram_timer_enable == RAM_TIMER_ENABLE_VALUE{
            match self.ram_rtc_select{
                0..=3=>{
                    let internal_address = self.ram_rtc_select as usize * RAM_BANK_SIZE as usize +  address as usize;
                    let address = get_external_ram_valid_address(internal_address, &self.ram);
                    self.ram[address] = value;
                },
                0x8..=0xC=>self.rtc_registers[(self.ram_rtc_select - 8) as usize] = value,
                _=>{}
            }
        }
    }
    
    #[cfg(feature = "dbg")]
    fn get_bank_number(&self)->u16 { self.get_current_rom_bank() as u16 }
}

impl<'a> Mbc3<'a>{
    pub fn new(program:&'a[u8], battery:bool, ram:Option<&'static mut[u8]>)->Self{
        let ram = init_ram(program[MBC_RAM_SIZE_LOCATION], ram);
        return Self{
            current_bank:0,
            battery:battery,
            latch_clock_data:0,
            program:program,
            ram,
            ram_rtc_select:0,
            ram_timer_enable:0,
            rtc_registers:[0;RTC_REGISTERS_COUNT]
        };
    }

    fn get_current_rom_bank(&self)->u8{
        //discard last bit as this register is 7 bits long
        let mut value = self.current_bank & !BIT_7_MASK;
        if value == 0{
            value = 1;
        }

        return value;
    }
}