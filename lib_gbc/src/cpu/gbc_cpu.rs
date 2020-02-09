pub struct GbcCpu {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub stack_pointer: u16,
    pub program_counter: u16,
}

impl Default for GbcCpu {
    fn default() -> GbcCpu {
        GbcCpu {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            stack_pointer: 0,
            program_counter: 0,
        }
    }
}

impl GbcCpu {
    pub fn af(&self) -> u16 {
        let mut value: u16 = self.a as u16;
        value <<= 8;
        value += self.f as u16;
        return value;
    }

    pub fn bc(&self) -> u16 {
        let mut value: u16 = self.b as u16;
        value <<= 8;
        value += self.c as u16;
        return value;
    }

    pub fn de(&self) -> u16 {
        let mut value: u16 = self.d as u16;
        value <<= 8;
        value += self.e as u16;
        return value;
    }

    pub fn hl(&self) -> u16 {
        let mut value: u16 = self.h as u16;
        value <<= 8;
        value += self.l as u16;
        return value;
    }

    pub fn get_register(&mut self, register_index: u8) -> &mut u8 {
        return match register_index {
            0b000 => &mut self.b,
            0b001 => &mut self.c,
            0b010 => &mut self.d,
            0b011 => &mut self.e,
            0b100 => &mut self.h,
            0b101 => &mut self.l,
            0b111 => &mut self.a,
            _ => std::panic!("No matching register for:{}", register_index)
        };
    }

    pub fn get_16bit_register(&mut self, register_index:u8)-> u16{
        return match register_index{
            0b00=>self.bc(),
            0b01=>self.de(),
            0b10=>self.hl(),
            0b11=>self.stack_pointer,
            _=>std::panic!("no matching 16bit register for:{}",register_index)
        };
    }

    pub fn set_16bit_register(&mut self, register_index:u8, value:u16){
        let first:u8 = (value & 0xFF) as u8;
        let second:u8 = ((value & 0xFF00) >> 8) as u8;
        match register_index{
            0b00=> {
                self.b = first;
                self.c = second;
            }
            0b01=>{
                self.d = first;
                self.e = second;
            }
            0b10=>{
                self.h = first;
                self.l = second;
            }
            0b11=>self.stack_pointer = value,
            _=>std::panic!("no matching 16bit register for:{}",register_index)
        };
    }

    pub fn inc_hl(&mut self){
        if self.l < 0xFF{
            self.l += 1;
        }
        else if self.h < 0xFF{
            self.h += 1;
            self.l = 0;
        }
        else{
            self.l = 0;
            self.h = 0;
        }
    }

    pub fn dec_hl(&mut self){
        if self.l > 0{
            self.l -= 1;
        }
        else if self.h > 0{
            self.h -= 1;
            self.l = 0xFF;
        }
        else{
            self.l = 0xFF;
            self.h = 0xFF;
        }
    }
}
