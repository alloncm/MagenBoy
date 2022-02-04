use crate::utils::bit_masks::*;

pub struct GbTimer{
    pub system_counter:u16,
    pub tima_overflow:bool,

    pub tima_register:u8,
    pub tma_register:u8,
    pub tac_tegister:u8,

    last_and_result:bool,
    reload_cooldown_counter:u8
}

impl Default for GbTimer{
    fn default() -> Self {
        GbTimer{
            system_counter:0,
            tima_register:0,
            tma_register:0,
            tac_tegister:0,
            last_and_result: false,
            reload_cooldown_counter: 0,
            tima_overflow:false
        }
    }
}

impl GbTimer{
    pub fn cycle(&mut self, m_cycles:u32, if_register:&mut u8)->u32{
        let (timer_interval, timer_enable) = self.get_timer_controller_data();

        for _ in 0..m_cycles * 4{
            if timer_enable && self.tima_overflow{
                self.reload_cooldown_counter += 1;
                if self.reload_cooldown_counter >= 4{
                    self.reload_cooldown_counter = 0;

                    *if_register |= BIT_2_MASK;
                    self.tima_register = self.tma_register;
                    self.tima_overflow = false;
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

            let current_and_result = bit_value && timer_enable;
            if !current_and_result && self.last_and_result{
                let(value, overflow) = self.tima_register.overflowing_add(1);
                self.tima_register = value;
                self.tima_overflow = overflow;
                self.reload_cooldown_counter = 0;
            }
            self.last_and_result = current_and_result;
        }

        let t_cycles_to_next_timer_event = match timer_interval{
            0b00=>0x100 - (self.system_counter & 0xFF),
            0b01=>0b1000 - (self.system_counter & 0b111),
            0b10=>0b10_0000 - (self.system_counter & 0b1_1111),
            0b11=>0b1000_0000 - (self.system_counter & 0b111_1111),
            _=>std::panic!("error ")
        };

        return (t_cycles_to_next_timer_event >> 2) as u32 + 1;
    }

    fn get_timer_controller_data(&self)->(u8, bool){
        let timer_enable:bool = self.tac_tegister & BIT_2_MASK != 0;

        return (self.tac_tegister & 0b11, timer_enable);
    }
}