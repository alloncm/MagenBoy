use crate::cpu::gb_cpu::GbCpu;
use crate::mmu::memory::Memory;
use crate::mmu::gbc_mmu::GbcMmu;
use crate::mmu::io_ports::DIV_REGISTER_INDEX;
use crate::ppu::gbc_ppu::GbcPpu;
use crate::utils::bit_masks::*;
use crate::utils::memory_registers::*;
use crate::utils::colors::*;
use crate::utils::color::Color;
use super::interrupts_handler::InterruptsHandler;
use crate::keypad::joypad::Joypad;
use crate::keypad::button::Button;
use crate::ppu::ppu_state::PpuState;


const DMA_SIZE:u16 = 0xA0;
const DMA_DEST:u16 = 0xFE00;
const LY_INTERRUPT_VALUE:u8 = 144;
const WX_OFFSET:u8 = 7;

pub struct RegisterHandler{
    timer_clock_interval_counter:u32,
    v_blank_triggered:bool,
    dma_cycle_counter:u16
}

impl Default for RegisterHandler{
    fn default()->Self{
        RegisterHandler{
            timer_clock_interval_counter: 0,
            v_blank_triggered:false,
            dma_cycle_counter:0
        }
    }
}

impl RegisterHandler{

    //TODO: update the rest of the function to use the cycles (I think only timer)
    pub fn update_registers_state(&mut self, memory: &mut GbcMmu, cpu:&mut GbCpu, ppu:&mut GbcPpu, interrupts_handler:&mut InterruptsHandler,joypad:&Joypad, m_cycles:u8){
        let interupt_enable = memory.read(IE_REGISTER_ADDRESS);
        let mut interupt_flag = memory.read(IF_REGISTER_ADDRESS);

        Self::handle_joypad_register(memory.read(JOYP_REGISTER_ADDRESS),memory, joypad);
        self.handle_ly_register(memory, ppu, &mut interupt_flag);
        Self::handle_lcdcontrol_register(memory.read(LCDC_REGISTER_ADDRESS), ppu);
        self.handle_lcd_status_register(memory.read(STAT_REGISTER_ADDRESS), interrupts_handler, memory, ppu, &mut interupt_flag);
        Self::handle_scroll_registers(memory.read(SCX_REGISTER_ADDRESS), memory.read(SCY_REGISTER_ADDRESS), ppu);
        Self::handle_vrambank_register(memory.read(VBK_REGISTER_ADDRESS), memory, cpu);
        Self::handle_switch_mode_register(memory.read(KEYI_REGISTER_ADDRESS), memory, cpu);
        Self::handle_wrambank_register(memory.read(SVBK_REGISTER_ADDRESS), memory);
        self.handle_dma_transfer_register(memory.read(DMA_REGISTER_ADDRESS), memory,m_cycles);
        Self::handle_bootrom_register(memory.read(BOOT_REGISTER_ADDRESS), memory);
        Self::handle_bg_pallet_register(memory.read(BGP_REGISTER_ADDRESS), &mut ppu.bg_color_mapping);
        Self::handle_obp_pallet_register(memory.read(OBP0_REGISTER_ADDRESS), &mut ppu.obj_color_mapping0);
        Self::handle_obp_pallet_register(memory.read(OBP1_REGISTER_ADDRESS), &mut ppu.obj_color_mapping1);
        self.handle_divider_register(memory,m_cycles);
        self.handle_timer_counter_register(memory.read(TIMA_REGISTER_ADDRESS), memory, &mut interupt_flag, m_cycles);
        Self::handle_wy_register(memory.read(WY_REGISTER_ADDRESS), ppu);
        Self::handle_wx_register(memory.read(WX_REGISTER_ADDRESS), ppu);

        //This should be last cause it updated the interupt values
        Self::handle_intreput_registers(interupt_enable, interupt_flag, cpu);

        memory.write(IF_REGISTER_ADDRESS, interupt_flag);
        memory.write(IE_REGISTER_ADDRESS, interupt_enable);
    }

    fn handle_intreput_registers(enable:u8, flag:u8, cpu:&mut GbCpu){
        cpu.interupt_enable = enable;
        cpu.interupt_flag = flag;
    }

    fn handle_bg_pallet_register(register:u8, pallet:&mut [Color;4] ){
        pallet[0] = Self::get_matching_color(register&0b00000011);
        pallet[1] = Self::get_matching_color((register&0b00001100)>>2);
        pallet[2] = Self::get_matching_color((register&0b00110000)>>4);
        pallet[3] = Self::get_matching_color((register&0b11000000)>>6);
    }

    fn handle_lcd_status_register(&mut self, mut register:u8, interrupts_handler:&mut InterruptsHandler, memory:&mut GbcMmu, ppu:&GbcPpu, if_register:&mut u8){
        let ly = memory.read(LY_REGISTER_ADDRESS);
        let lyc = memory.read(LYC_REGISTER_ADDRESS);

        interrupts_handler.h_blank_interrupt = register & BIT_3_MASK != 0;
        interrupts_handler.v_blank_interrupt = register & BIT_4_MASK != 0;
        interrupts_handler.oam_search = register & BIT_5_MASK != 0;
        interrupts_handler.coincidence_interrupt = register & BIT_6_MASK != 0;


        if register & 0b11 != ppu.state as u8{
            let mut lcd_stat_interrupt:bool = false;

            if ly == lyc{
                register |= BIT_2_MASK;
                if interrupts_handler.coincidence_interrupt && ppu.state as u8 == PpuState::OamSearch as u8{
                    lcd_stat_interrupt = true;
                }
            }
            else{
                register &= !BIT_2_MASK;
            }
            
            memory.ppu_state = ppu.state;
            //clears the 2 lower bits
            register = (register >> 2)<<2;
            register |= ppu.state as u8;

            match ppu.state{
                PpuState::OamSearch=>{
                    if interrupts_handler.oam_search{
                        lcd_stat_interrupt = true;
                    }
                },
                PpuState::Hblank=>{
                    if interrupts_handler.h_blank_interrupt{
                        lcd_stat_interrupt = true;
                    }
                },
                PpuState::Vblank=>{
                    if interrupts_handler.v_blank_interrupt{
                        lcd_stat_interrupt = true;
                    }
                },
                _=>{}
            }

            if lcd_stat_interrupt{
                *if_register |= BIT_1_MASK;
            }
        }

        memory.io_ports.write_unprotected(STAT_REGISTER_ADDRESS - 0xFF00, register);
    }

    fn handle_obp_pallet_register(register:u8, pallet:&mut [Option<Color>;4] ){
        pallet[0] = None;
        pallet[1] = Some(Self::get_matching_color((register&0b00001100)>>2));
        pallet[2] = Some(Self::get_matching_color((register&0b00110000)>>4));
        pallet[3] = Some(Self::get_matching_color((register&0b11000000)>>6));
    }

    fn get_matching_color(number:u8)->Color{
        return match number{
            0b00=>WHITE,
            0b01=>LIGHT_GRAY,
            0b10=>DARK_GRAY,
            0b11=>BLACK,
            _=>std::panic!("no macthing color for color number: {}", number)
        };
    }
    
    fn handle_ly_register(&mut self, memory:&mut dyn Memory, ppu:&GbcPpu, if_register:&mut u8){
        if ppu.current_line_drawn >= LY_INTERRUPT_VALUE && !self.v_blank_triggered{
            //V-Blank interrupt
            *if_register |= BIT_0_MASK;
            self.v_blank_triggered = true;
        }
        else if ppu.current_line_drawn < LY_INTERRUPT_VALUE{

            self.v_blank_triggered = false;
        }
        
        memory.write(LY_REGISTER_ADDRESS, ppu.current_line_drawn);        
    }
    

    fn handle_bootrom_register(register:u8, memory: &mut GbcMmu){
        memory.finished_boot = register == 1;
    }

    fn handle_lcdcontrol_register( register:u8, ppu:&mut GbcPpu){
        ppu.screen_enable = (register & BIT_7_MASK) != 0;
        ppu.window_tile_map_address = (register & BIT_6_MASK) != 0;
        ppu.window_enable = (register & BIT_5_MASK) != 0;
        ppu.window_tile_background_map_data_address = (register & BIT_4_MASK) != 0;
        ppu.background_tile_map_address = (register & BIT_3_MASK) != 0;
        ppu.sprite_extended = (register & BIT_2_MASK) != 0;
        ppu.sprite_enable = (register & BIT_1_MASK) != 0;
        ppu.background_enabled = (register & BIT_0_MASK) != 0;
    }

    fn handle_scroll_registers(scroll_x:u8, scroll_y:u8, ppu:&mut GbcPpu){
        ppu.background_scroll.x = scroll_x;
        ppu.background_scroll.y = scroll_y;
    }

    fn handle_vrambank_register( register:u8, memory: &mut GbcMmu, cpu:&mut GbCpu){
        if cpu.cgb_mode{
            memory.vram.set_bank(register & BIT_0_MASK);
        }
    }

    fn handle_switch_mode_register( register:u8, memory: &mut dyn Memory, cpu:&mut GbCpu){
        if register & BIT_0_MASK != 0{
            cpu.cgb_mode = !cpu.cgb_mode;
            let cgb_mask = (cpu.cgb_mode as u8) <<7;
            memory.write(0xFF4D, register | cgb_mask);
        }
    }

    fn handle_wrambank_register( register:u8, memory: &mut GbcMmu){
        let bank:u8 = register & 0b00000111;
        memory.ram.set_bank(bank);
    }

    fn handle_dma_transfer_register(&mut self, register:u8, mmu:&mut GbcMmu, m_cycles:u8){
        if mmu.dma_trasfer_trigger{
            let source:u16 = (register as u16) << 8;
            let cycles_to_run = std::cmp::min(self.dma_cycle_counter + m_cycles as u16, DMA_SIZE);
            for i in self.dma_cycle_counter..cycles_to_run as u16{
                mmu.write(DMA_DEST + i, mmu.read(source + i));
            }

            self.dma_cycle_counter += m_cycles as u16;
            
            if self.dma_cycle_counter >= DMA_SIZE{
                mmu.dma_trasfer_trigger = false;   
                self.dma_cycle_counter = 0;
            }
        }
    }

    fn handle_divider_register(&mut self, mmu:&mut GbcMmu, m_cycles:u8){
        const T_CYCLES_IN_M_CYCLE:u8 = 4;
        mmu.io_ports.system_counter = mmu.io_ports.system_counter.wrapping_add(m_cycles as u16 * T_CYCLES_IN_M_CYCLE as u16);
        mmu.io_ports.write_unprotected(DIV_REGISTER_INDEX, (mmu.io_ports.system_counter >> 8) as u8);
    }

    fn handle_timer_counter_register(&mut self, register:u8, memory:&mut dyn Memory, if_register:&mut u8, m_cycles_passed:u8){
        let (interval, enable) = Self::get_timer_controller_data(memory);

        if enable{
            self.timer_clock_interval_counter += m_cycles_passed as u32;

            if self.timer_clock_interval_counter >= interval{
                self.timer_clock_interval_counter -= interval as u32;

                let (mut value, overflow) = register.overflowing_add(1);

                if overflow{
                    *if_register |= BIT_2_MASK;
                    value = memory.read(TMA_REGISTER_ADDRESS);
                }

                memory.write(TIMA_REGISTER_ADDRESS, value);
            }
        }
    }
    
    fn get_timer_controller_data(memory: &mut dyn Memory)->(u32, bool){
        let timer_controller = memory.read(TAC_REGISTER_ADDRESS);
        let timer_enable:bool = timer_controller & BIT_2_MASK != 0;

        //those are the the number of m_cycles to wait bwtween each update
        let interval = match timer_controller & 0b11{
            0b00=>256,
            0b01=>4,
            0b10=>16,
            0b11=>64,
            _=>std::panic!("timer controller value is out of range")
        };

        return (interval, timer_enable);
    }

    fn handle_wy_register(register:u8, ppu:&mut GbcPpu){
        ppu.window_scroll.y = register;
    }

    fn handle_wx_register(register:u8, ppu:&mut GbcPpu){
        if register < WX_OFFSET{
            ppu.window_scroll.x = 0;
        }
        else{
            ppu.window_scroll.x = register - WX_OFFSET;
        }
    }

    // This register stores key pressed as 0 (unset bit) and not 1 (set bit) 
    // so this function will beahve accordingly
    fn handle_joypad_register(mut state:u8,memory:&mut GbcMmu,joypad:&Joypad){
        let buttons = (state & BIT_5_MASK) == 0;
        let directions = (state & BIT_4_MASK) == 0;

        if buttons{
            Self::set_bit(&mut state, 0, !joypad.buttons[Button::A as usize]);
            Self::set_bit(&mut state, 1, !joypad.buttons[Button::B as usize]);
            Self::set_bit(&mut state, 2, !joypad.buttons[Button::Select as usize]);
            Self::set_bit(&mut state, 3, !joypad.buttons[Button::Start as usize]);
        }
        if directions{
            Self::set_bit(&mut state, 0, !joypad.buttons[Button::Right as usize]);
            Self::set_bit(&mut state, 1, !joypad.buttons[Button::Left as usize]);
            Self::set_bit(&mut state, 2, !joypad.buttons[Button::Up as usize]);
            Self::set_bit(&mut state, 3, !joypad.buttons[Button::Down as usize]);
        }

        memory.io_ports.write_unprotected(JOYP_REGISTER_ADDRESS - 0xFF00, state);
    }

    fn set_bit(value:&mut u8, bit_number:u8, set:bool){
        let mask = 1 << bit_number;
        if set{
            *value |= mask;
        }
        else{
            let inverse_mask = !mask;
            *value &= inverse_mask;
        }
    }
}