use crate::cpu::gbc_cpu::GbcCpu;
use crate::utils::bit_masks::*;
use crate::opcodes::opcodes_utils::push;
use crate::mmu::memory::Memory;

const V_BLACK_INTERRUPT_ADDERESS:u16    = 0x40;
const LCD_STAT_INTERRUPT_ADDERESS:u16   = 0x48;
const TIMER_INTERRUPT_ADDERESS:u16      = 0x50;
const SRIAL_INTERRUPT_ADDERESS:u16      = 0x58;
const JOYPAD_INTERRUPT_ADDERESS:u16     = 0x60;


pub fn handle_interrupts(cpu:&mut GbcCpu, memory:&mut dyn Memory){
    if cpu.mie{
        if cpu.interupt_flag & BIT_0_MASK != 0 && cpu.interupt_enable & BIT_0_MASK != 0{
            prepare_for_interut(cpu, BIT_0_MASK, V_BLACK_INTERRUPT_ADDERESS, memory);
        }
        else if cpu.interupt_flag & BIT_1_MASK != 0 && cpu.interupt_enable & BIT_1_MASK != 0{
            prepare_for_interut(cpu, BIT_1_MASK, LCD_STAT_INTERRUPT_ADDERESS, memory);
        }
        else if cpu.interupt_flag & BIT_2_MASK != 0 && cpu.interupt_enable & BIT_2_MASK != 0{
            prepare_for_interut(cpu, BIT_2_MASK, TIMER_INTERRUPT_ADDERESS, memory);
        }
        else if cpu.interupt_flag & BIT_3_MASK != 0 && cpu.interupt_enable & BIT_3_MASK != 0{
            prepare_for_interut(cpu, BIT_3_MASK, SRIAL_INTERRUPT_ADDERESS, memory);
        }
        else if cpu.interupt_flag & BIT_4_MASK != 0 && cpu.interupt_enable & BIT_4_MASK != 0{
            prepare_for_interut(cpu, BIT_4_MASK, JOYPAD_INTERRUPT_ADDERESS, memory);
        }
    }
}

fn prepare_for_interut(cpu:&mut GbcCpu, interupt_bit:u8, address:u16, memory:&mut dyn Memory){
    //reseting the interupt bit
    cpu.interupt_flag ^= interupt_bit;
    //reseting MIE register
    cpu.mie = false;
    //pushing PC
    push(cpu, memory, cpu.program_counter);
    //jumping to the interupt address
    cpu.program_counter = address;
}