use crate::utils::bit_masks::*;

const V_BLANK_INTERRUPT_ADDERESS:u16    = 0x40;
const LCD_STAT_INTERRUPT_ADDERESS:u16   = 0x48;
const TIMER_INTERRUPT_ADDERESS:u16      = 0x50;
const SRIAL_INTERRUPT_ADDERESS:u16      = 0x58;
const JOYPAD_INTERRUPT_ADDERESS:u16     = 0x60;

pub enum InterruptRequest{
    Interrupt(u16),
    Unhalt,
    None
}

pub struct InterruptsHandler{
    pub interrupt_flag:u8,
    pub interrupt_enable_flag:u8,
    ei_triggered:bool // use to delay the interrupt execution by one instruction in a special case
}

impl Default for InterruptsHandler{
    fn default()->Self{
        InterruptsHandler{
            interrupt_flag:0,
            interrupt_enable_flag:0,
            ei_triggered:false
        }
    }
}

impl InterruptsHandler{
    pub fn handle_interrupts(&mut self, master_interrupt_enable:bool, stat_register:u8)->InterruptRequest{
        let mut interrupt_request = InterruptRequest::None;

        //there is a delay of one instruction cause there is this delay since EI opcode is called untill the interrupt could happen
        if master_interrupt_enable && self.ei_triggered {
            // The order is the interrupt priority of the interrupts

            if self.interrupt_flag & BIT_0_MASK != 0 && self.interrupt_enable_flag & BIT_0_MASK != 0{
                interrupt_request = self.prepare_for_interrupt( BIT_0_MASK, V_BLANK_INTERRUPT_ADDERESS);
            }
            // Checking those STAT register bits for the STAT interrupts requests
            else if self.interrupt_flag & BIT_1_MASK != 0 && self.interrupt_enable_flag & BIT_1_MASK != 0 && (stat_register & 0b111_1000) != 0{
                interrupt_request = self.prepare_for_interrupt(BIT_1_MASK, LCD_STAT_INTERRUPT_ADDERESS);
            }
            else if self.interrupt_flag & BIT_2_MASK != 0 && self.interrupt_enable_flag & BIT_2_MASK != 0{
                interrupt_request = self.prepare_for_interrupt(BIT_2_MASK, TIMER_INTERRUPT_ADDERESS);
            }
            else if self.interrupt_flag & BIT_3_MASK != 0 && self.interrupt_enable_flag & BIT_3_MASK != 0{
                interrupt_request = self.prepare_for_interrupt(BIT_3_MASK, SRIAL_INTERRUPT_ADDERESS);
            }
            else if self.interrupt_flag & BIT_4_MASK != 0 && self.interrupt_enable_flag & BIT_4_MASK != 0{
                interrupt_request = self.prepare_for_interrupt(BIT_4_MASK, JOYPAD_INTERRUPT_ADDERESS);
            }
        }
        else {
            // if anding them is not zero there is at least one interrupt pending
            if (self.interrupt_enable_flag & self.interrupt_flag) & 0b1_1111 != 0{
                interrupt_request = InterruptRequest::Unhalt;
            }
        }

        self.ei_triggered = master_interrupt_enable;
        return interrupt_request;
    }

    fn prepare_for_interrupt(&mut self, interupt_bit:u8, address:u16)->InterruptRequest{
        //reseting the interupt bit
        self.interrupt_flag &= !interupt_bit;
        
        return InterruptRequest::Interrupt(address);
    }
}