use crate::mmu::{interrupts_handler::InterruptRequest, Memory};

use super::register::Reg;
use super::flag::Flag;

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
    pub double_speed:bool
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
            double_speed:false
        }
    }
}

impl GbCpu {
    pub fn execute_interrupt_request(&mut self, memory:&mut impl Memory, ir: InterruptRequest)->u8{
        match ir{
            InterruptRequest::Unhalt=>{
                self.halt = false;
                memory.set_halt(false);
            },
            InterruptRequest::Interrupt(address)=>return self.prepare_for_interrupt(memory, address),
            InterruptRequest::None=>{}
        }

        return 0;
    }

    fn prepare_for_interrupt(&mut self, memory: &mut impl Memory, address: u16)->u8{
        //reseting MIE register
        self.mie = false;
        //pushing PC
        super::opcodes::opcodes_utils::push(self, memory, self.program_counter);
        //jumping to the interupt address
        self.program_counter = address;
        //unhalting the CPU
        self.halt = false;

        // 5 cycles - 2 pushing pc to memory, 3 internal operation
        return 3;
    }

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
        *self.hl.value_mut() = self.hl.value().wrapping_add(1);
    }

    pub fn dec_hl(&mut self){
        *self.hl.value_mut() = self.hl.value().wrapping_sub(1);
    }
}
