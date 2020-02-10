use crate::cpu::register::Reg;

pub struct GbcCpu {
    pub af: Reg,
    pub bc: Reg,
    pub de: Reg,
    pub hl: Reg,
    pub stack_pointer: u16,
    pub program_counter: u16,
}

impl Default for GbcCpu {
    fn default() -> GbcCpu {
        GbcCpu {
            af: Reg::default(),
            bc: Reg::default(),
            de: Reg::default(),
            hl: Reg::default(),
            stack_pointer: 0,
            program_counter: 0
        }
    }
}

impl GbcCpu {


    pub fn inc_hl(&mut self){
        self.hl.value = self.hl.value.wrapping_add(1);
    }

    pub fn dec_hl(&mut self){
        self.hl.value = self.hl.value.wrapping_sub(1);
    }
}
