use crate::{cpu::gb_cpu::GbCpu, ppu::gb_ppu::GbPpu, utils::memory_registers::IE_REGISTER_ADDRESS};
use crate::utils::{
    bit_masks::*,
    memory_registers::IF_REGISTER_ADDRESS
};
use crate::cpu::opcodes::opcodes_utils::push;
use crate::mmu::memory::Memory;

const V_BLANK_INTERRUPT_ADDERESS:u16    = 0x40;
const LCD_STAT_INTERRUPT_ADDERESS:u16   = 0x48;
const TIMER_INTERRUPT_ADDERESS:u16      = 0x50;
const SRIAL_INTERRUPT_ADDERESS:u16      = 0x58;
const JOYPAD_INTERRUPT_ADDERESS:u16     = 0x60;

pub struct InterruptsHandler{
    ei_triggered:bool
}

impl Default for InterruptsHandler{
    fn default()->Self{
        InterruptsHandler{
            ei_triggered:false
        }
    }
}

impl InterruptsHandler{

    pub fn handle_interrupts(&mut self, cpu:&mut GbCpu,ppu:&mut GbPpu, memory:&mut impl Memory)->u8{
        //this is delatey by one instruction cause there is this delay since EI opcode is called untill the interrupt could happen
        
        let interupt_flag = memory.read(IF_REGISTER_ADDRESS);
        let interupt_enable = memory.read(IE_REGISTER_ADDRESS);

        if cpu.mie && self.ei_triggered{
            if interupt_flag & BIT_0_MASK != 0 && interupt_enable & BIT_0_MASK != 0{
                return Self::prepare_for_interut(cpu, BIT_0_MASK, V_BLANK_INTERRUPT_ADDERESS, memory, interupt_flag);
            }
            if interupt_flag & BIT_1_MASK != 0 && interupt_enable & BIT_1_MASK != 0 && 
            (ppu.v_blank_interrupt_request || ppu.oam_search_interrupt_request || ppu.h_blank_interrupt_request || ppu.coincidence_interrupt_request){
                return Self::prepare_for_interut(cpu, BIT_1_MASK, LCD_STAT_INTERRUPT_ADDERESS, memory, interupt_flag);
            }
            if interupt_flag & BIT_2_MASK != 0 && interupt_enable & BIT_2_MASK != 0{
                return Self::prepare_for_interut(cpu, BIT_2_MASK, TIMER_INTERRUPT_ADDERESS, memory, interupt_flag);
            }
            if interupt_flag & BIT_3_MASK != 0 && interupt_enable & BIT_3_MASK != 0{
                return Self::prepare_for_interut(cpu, BIT_3_MASK, SRIAL_INTERRUPT_ADDERESS, memory, interupt_flag);
            }
            if interupt_flag & BIT_4_MASK != 0 && interupt_enable & BIT_4_MASK != 0{
                return Self::prepare_for_interut(cpu, BIT_4_MASK, JOYPAD_INTERRUPT_ADDERESS, memory, interupt_flag);
            }
        }
        else if cpu.halt{
            for i in 0..5{
                let mask = 1 << i;
                if interupt_flag & mask != 0 && interupt_enable & mask != 0{
                    cpu.halt = false;
                }
            }
        }

        self.ei_triggered = cpu.mie;

        //no cycles passed
        return 0;
    }

    fn prepare_for_interut(cpu:&mut GbCpu, interupt_bit:u8, address:u16, memory:&mut impl Memory, mut interupt_flag:u8)->u8{
        //reseting the interupt bit
        interupt_flag &= !interupt_bit;
        memory.write(IF_REGISTER_ADDRESS, interupt_flag);
        //reseting MIE register
        cpu.mie = false;
        //pushing PC
        push(cpu, memory, cpu.program_counter);
        //jumping to the interupt address
        cpu.program_counter = address;
        //unhalting the CPU
        cpu.halt = false;

        //cycles passed
        return 5;
    }
}