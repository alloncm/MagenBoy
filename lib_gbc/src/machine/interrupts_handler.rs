use crate::cpu::gbc_cpu::GbcCpu;
use crate::utils::bit_masks::*;

const V_BLACK_INTERRUPT_ADDERESS:u16    = 0x40;
const LCD_STAT_INTERRUPT_ADDERESS:u16   = 0x48;
const TIMER_INTERRUPT_ADDERESS:u16      = 0x50;
const SRIAL_INTERRUPT_ADDERESS:u16      = 0x58;
const JOYPAD_INTERRUPT_ADDERESS:u16     = 0x60;


pub fn handle_interrupts(cpu:&mut GbcCpu){
    if cpu.mie{
        if cpu.interupt_flag & BIT_0_MASK != 0 && cpu.interupt_enable & BIT_0_MASK != 0{
            cpu.program_counter = V_BLACK_INTERRUPT_ADDERESS;
            cpu.interupt_flag &= !BIT_0_MASK;
        }
        else if cpu.interupt_flag & BIT_1_MASK != 0 && cpu.interupt_enable & BIT_1_MASK != 0{
            cpu.program_counter = LCD_STAT_INTERRUPT_ADDERESS;
            cpu.interupt_flag &= !BIT_1_MASK;
        }
        else if cpu.interupt_flag & BIT_2_MASK != 0 && cpu.interupt_enable & BIT_2_MASK != 0{
            cpu.program_counter = TIMER_INTERRUPT_ADDERESS;
            cpu.interupt_flag &= !BIT_2_MASK;
        }
        else if cpu.interupt_flag & BIT_3_MASK != 0 && cpu.interupt_enable & BIT_3_MASK != 0{
            cpu.program_counter = SRIAL_INTERRUPT_ADDERESS;
            cpu.interupt_flag &= !BIT_3_MASK;
        }
        else if cpu.interupt_flag & BIT_4_MASK != 0 && cpu.interupt_enable & BIT_4_MASK != 0{
            cpu.program_counter = JOYPAD_INTERRUPT_ADDERESS;
            cpu.interupt_flag &= !BIT_4_MASK;
        }
    }
}