use crate::{mmu::memory::UnprotectedMemory, utils::{bit_masks::*, memory_registers::*}};

pub struct GbTimer{
    pub system_counter:u16,

    last_and_result:bool,
    reload_cooldown_counter:u8
}

impl Default for GbTimer{
    fn default() -> Self {
        GbTimer{
            system_counter:0,
            last_and_result: false,
            reload_cooldown_counter: 0
        }
    }
}

impl GbTimer{
    pub fn cycle(&mut self, memory:&mut impl UnprotectedMemory, m_cycles:u8){
        let mut tima_register = memory.read_unprotected(TIMA_REGISTER_ADDRESS);
        let (timer_interval, timer_enable) = Self::get_timer_controller_data(memory);

        for _ in 0..m_cycles * 4{
            if tima_register == 0{
                if self.reload_cooldown_counter == 4{
                    self.reload_cooldown_counter = 0;

                    let mut if_register = memory.read_unprotected(IF_REGISTER_ADDRESS);
                    if_register |= BIT_2_MASK;
                    memory.write_unprotected(IF_REGISTER_ADDRESS, if_register);
                    tima_register = memory.read_unprotected(TMA_REGISTER_ADDRESS);
                }
                else{
                    self.reload_cooldown_counter += 1;
                }
            }

            self.system_counter = self.system_counter.wrapping_add(1);
            let bit_value:bool = match timer_interval{
                0b00=>(self.system_counter & BIT_9_MASK) != 0,
                0b01=>(self.system_counter & BIT_3_MASK as u16) != 0,
                0b10=>(self.system_counter & BIT_5_MASK as u16) != 0,
                0b11=>(self.system_counter & BIT_7_MASK as u16) != 0,
                _=> std::panic!("bad timer interval vlaue: {}", timer_interval)
            };

            let current_and_result = bit_value & timer_enable;
            if !current_and_result && self.last_and_result{
                tima_register = tima_register.wrapping_add(1);
            }
        }

        memory.write_unprotected(DIV_REGISTER_ADDRESS, (self.system_counter >> 8) as u8);
        memory.write_unprotected(TIMA_REGISTER_ADDRESS, tima_register);

        // old implementation

        // self.system_counter = self.system_counter.wrapping_add(m_cycles as u16 * 4 as u16);
        // memory.write_unprotected(DIV_REGISTER_ADDRESS, (self.system_counter >> 8) as u8);


        // let register = memory.read_unprotected(TIMA_REGISTER_ADDRESS);
        // let (interval, enable) = Self::get_timer_controller_data(memory);

        // if enable{
        //     self.timer_clock_interval_counter += m_cycles as u16;

        //     if self.timer_clock_interval_counter >= interval{
        //         self.timer_clock_interval_counter -= interval as u16;

        //         let (mut value, overflow) = register.overflowing_add(1);

        //         if overflow{
        //             let mut if_register = memory.read_unprotected(IF_REGISTER_ADDRESS);
        //             if_register |= BIT_2_MASK;
        //             memory.write_unprotected(IF_REGISTER_ADDRESS, if_register);

        //             value = memory.read_unprotected(TMA_REGISTER_ADDRESS);
        //         }
        //         //println!("ignore write tima");

        //         memory.write_unprotected(TIMA_REGISTER_ADDRESS, value);
        //     }
        // }
        
    }

    fn get_timer_controller_data(memory: &mut impl UnprotectedMemory)->(u8, bool){
        let timer_controller = memory.read_unprotected(TAC_REGISTER_ADDRESS);
        let timer_enable:bool = timer_controller & BIT_2_MASK != 0;

        //those are the the number of m_cycles to wait bwtween each update
        let interval = match timer_controller & 0b11{
            0b00=>256,
            0b01=>4,
            0b10=>16,
            0b11=>64,
            _=>std::panic!("timer controller value is out of range")
        };

        return (timer_controller & 0b11, timer_enable);
    }
}