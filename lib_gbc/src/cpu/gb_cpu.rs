use super::register::Reg;

pub enum Flag{
    Carry = 0b00010000,
    HalfCarry = 0b00100000,
    Subtraction = 0b01000000,
    Zero = 0b10000000
}

pub struct GbCpu {
    pub af: Reg,
    pub bc: Reg,
    pub de: Reg,
    pub hl: Reg,
    pub stack_pointer: u16,
    pub program_counter: u16,
    pub mie: bool,
    pub halt:bool,
    pub stop:bool,
    pub cgb_mode:bool,
    pub double_speed:bool,
    pub interupt_enable:u8,
    pub interupt_flag:u8
}

impl Default for GbCpu {
    fn default() -> Self {
        GbCpu {
            af: Reg::new(0xFFF0),
            bc: Reg::default(),
            de: Reg::default(),
            hl: Reg::default(),
            stack_pointer: 0,
            program_counter: 0,
            mie: false,
            halt:false,
            stop:false,
            cgb_mode:false,
            double_speed:false,
            interupt_enable:0,
            interupt_flag:0
        }
    }
}

impl GbCpu {
    pub fn set_flag(&mut self, flag:Flag){
        *self.af.low() |= flag as u8;
    }

    pub fn unset_flag(&mut self, flag:Flag){
        let f = !(flag as u8);
        *self.af.low() &= f;
    }

    pub fn set_by_value(&mut self, flag:Flag, value:bool){
        if value{
            self.set_flag(flag);
        }
        else{
            self.unset_flag(flag);
        }
    }

    pub fn get_flag(&mut self, flag:Flag)->bool{
        (*self.af.low() & flag as u8) != 0
    }

    pub fn inc_hl(&mut self){
        *self.hl.value() = (*self.hl.value()).wrapping_add(1);
    }

    pub fn dec_hl(&mut self){
        *self.hl.value() = (*self.hl.value()).wrapping_sub(1);
    }
}
