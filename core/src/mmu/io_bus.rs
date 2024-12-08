use crate::{
    apu::{audio_device::AudioDevice, gb_apu::GbApu, *}, 
    keypad::{joypad_handler::JoypadHandler, joypad_provider::JoypadProvider}, 
    machine::Mode, 
    ppu::{gb_ppu::GbPpu, gfx_device::GfxDevice}, 
    timer::{gb_timer::GbTimer, timer_register_updater::*}, utils::bit_masks::BIT_2_MASK
};
use super::{interrupts_handler::*, io_ports::*, oam_dma_controller::OamDmaController, vram_dma_controller::VramDmaController, external_memory_bus::ExternalMemoryBus, access_bus::AccessBus};

pub const IO_PORTS_SIZE:usize = 0x80;
const WAVE_RAM_START_INDEX:u16 = 0x30;
const WAVE_RAM_END_INDEX:u16 = 0x3F;

pub struct IoBus<AD:AudioDevice, GFX:GfxDevice, JP:JoypadProvider>{
    pub apu: GbApu<AD>,
    pub timer: GbTimer,
    pub ppu:GbPpu<GFX>,
    pub oam_dma_controller:OamDmaController,
    pub vram_dma_controller: VramDmaController,
    pub interrupt_handler:InterruptsHandler,
    pub joypad_handler: JoypadHandler<JP>,
    pub speed_switch_register:u8,
    mode: Mode,
    key0_register:u8,
    boot_finished:bool,

    speed_cycle_reminder:u8,

    apu_cycles_counter:u32,
    ppu_cycles:u32,
    timer_cycles:u32,

    timer_event_cycles:u32,
    apu_event_cycles:u32,

    // PPU can be completely off and not operate at all Im using an option
    // where None indicates the PPU is off
    ppu_event:Option<u32>,
}

impl<AD:AudioDevice, GFX:GfxDevice, JP:JoypadProvider> IoBus<AD, GFX, JP>{
    pub fn read(&mut self, address:u16)->u8 {

        match address{
            TAC_REGISTER_INDEX | DIV_REGISTER_INDEX | TIMA_REGISTER_INDEX=> self.cycle_timer(),
            NR10_REGISTER_INDEX..=WAVE_RAM_END_INDEX => self.cycle_apu(),
            LCDC_REGISTER_INDEX..=WX_REGISTER_INDEX => self.cycle_ppu(),
            _=>{}
        }

        return match address {
            //Timer
            TAC_REGISTER_INDEX=> self.timer.tac_tegister,
            DIV_REGISTER_INDEX=> get_div(&self.timer),
            TIMA_REGISTER_INDEX=> self.timer.tima_register,
            //Interrupts handler
            IF_REGISTER_INDEX => self.interrupt_handler.interrupt_flag,
            //APU
            NR10_REGISTER_INDEX=> self.apu.sweep_tone_channel.sample_producer.sweep.as_mut().unwrap().nr10_register,
            NR11_REGISTER_INDEX=> (self.apu.sweep_tone_channel.sample_producer.wave_duty << 6) | 0b0011_1111,
            NR12_REGISTER_INDEX => self.apu.sweep_tone_channel.sample_producer.envelop.nrx2_register,
            NR13_REGISTER_INDEX=> 0xFF,
            NR14_REGISTER_INDEX=> ((self.apu.sweep_tone_channel.length_enable as u8) << 6 ) | 0b1011_1111,
            0x15 => 0xFF, //Not used
            NR21_REGISTER_INDEX=> (self.apu.tone_channel.sample_producer.wave_duty << 6) | 0b0011_1111,
            NR22_REGISTER_INDEX=> self.apu.tone_channel.sample_producer.envelop.nrx2_register,
            NR23_REGISTER_INDEX=> 0xFF,
            NR24_REGISTER_INDEX=> ((self.apu.tone_channel.length_enable as u8) << 6 ) | 0b1011_1111,
            NR30_REGISTER_INDEX=> ((self.apu.wave_channel.enabled as u8) << 7) | 0b0111_1111,
            NR31_REGISTER_INDEX=> 0xFF,
            NR32_REGISTER_INDEX=> (self.apu.wave_channel.sample_producer.volume << 5) | 0b1001_1111,
            NR33_REGISTER_INDEX=> 0xFF,
            NR34_REGISTER_INDEX=> ((self.apu.wave_channel.length_enable as u8) << 6 ) | 0b1011_1111,
            0x1F => 0xFF, //Not used
            NR41_REGISTER_INDEX=> 0xFF,
            NR42_REGISTER_INDEX=> self.apu.noise_channel.sample_producer.envelop.nrx2_register,
            NR43_REGISTER_INDEX=> self.apu.noise_channel.sample_producer.nr43_register,
            NR44_REGISTER_INDEX=> ((self.apu.wave_channel.length_enable as u8) << 6 ) | 0b1011_1111,
            NR50_REGISTER_INDEX=> self.apu.nr50_register,
            NR51_REGISTER_INDEX=> self.apu.nr51_register,
            NR52_REGISTER_INDEX=> get_nr52(&self.apu),
            0x27..=0x2F => 0xFF, //Not used
            WAVE_RAM_START_INDEX..=WAVE_RAM_END_INDEX => get_wave_ram(&self.apu.wave_channel, address),
            //PPU
            LCDC_REGISTER_INDEX=>self.ppu.lcd_control,
            STAT_REGISTER_INDEX=> self.ppu.get_stat(),
            SCY_REGISTER_INDEX=> self.ppu.bg_pos.y,
            SCX_REGISTER_INDEX=> self.ppu.bg_pos.x,
            LY_REGISTER_INDEX=> self.ppu.ly_register,
            LYC_REGISTER_INDEX=> self.ppu.lyc_register,
            DMA_REGISTER_INDEX=> self.oam_dma_controller.get_dma_register(),
            BGP_REGISTER_INDEX=> self.ppu.bg_palette_register,
            OBP0_REGISTER_INDEX=> self.ppu.obj_pallete_0_register,
            OBP1_REGISTER_INDEX=> self.ppu.obj_pallete_1_register,
            WY_REGISTER_INDEX => self.ppu.window_pos.y,
            WX_REGISTER_INDEX=> self.ppu.get_wx_register(),
            //Joypad
            JOYP_REGISTER_INDEX => self.joypad_handler.get_register(),

            // CGB registers
            _ if self.mode == Mode::CGB => match address{
                VBK_REGISTER_INDEX =>self.ppu.vram.get_bank_reg(),
                KEY0_REGISTER_INDEX => self.key0_register,
                //GBC speed switch
                KEY1_REGISTER_INDEX =>self.speed_switch_register | 0b0111_1110,
                ORPI_REGISTER_INDEX => self.ppu.get_orpi(),
                // VRAM DMA
                HDMA5_REGISTER_INDEX =>self.vram_dma_controller.get_mode_length(),
                //Color ram
                BGPI_REGISTER_INDEX =>self.ppu.get_bgpi(),
                BGPD_REGISTER_INDEX =>self.ppu.get_bgpd(),
                OBPI_REGISTER_INDEX =>self.ppu.get_obpi(),
                OBPD_REGISTER_INDEX =>self.ppu.get_obpd(),
                _=>0xFF
            }
            _=>0xFF
        };
    }

    pub fn write(&mut self, address:u16, value:u8) {
        match address{
            DIV_REGISTER_INDEX | TIMA_REGISTER_INDEX | TMA_REGISTER_INDEX | TAC_REGISTER_INDEX => self.cycle_timer(),
            NR10_REGISTER_INDEX..=WAVE_RAM_END_INDEX => self.cycle_apu(),
            LCDC_REGISTER_INDEX..=WX_REGISTER_INDEX => self.cycle_ppu(),
            _=>{}
        }
        match address{
            //timer
            DIV_REGISTER_INDEX=> reset_div(&mut self.timer),
            TIMA_REGISTER_INDEX=> set_tima(&mut self.timer, value),
            TMA_REGISTER_INDEX=> set_tma(&mut self.timer, value),
            TAC_REGISTER_INDEX=> set_tac(&mut self.timer, value),
            //Interrut handler
            IF_REGISTER_INDEX => self.interrupt_handler.interrupt_flag = value,
            //APU
            NR10_REGISTER_INDEX=> set_nr10(&mut self.apu.sweep_tone_channel, value),
            NR11_REGISTER_INDEX=> set_nrx1(&mut self.apu.sweep_tone_channel, value),
            NR12_REGISTER_INDEX=> set_nrx2(&mut self.apu.sweep_tone_channel, value),
            NR13_REGISTER_INDEX=> set_nrx3(&mut self.apu.sweep_tone_channel, value),
            NR14_REGISTER_INDEX=> set_nr14(&mut self.apu.sweep_tone_channel, &self.apu.frame_sequencer, value),
            NR21_REGISTER_INDEX=> set_nrx1(&mut self.apu.tone_channel, value),
            NR22_REGISTER_INDEX=> set_nrx2(&mut self.apu.tone_channel, value),
            NR23_REGISTER_INDEX=> set_nrx3(&mut self.apu.tone_channel, value),
            NR24_REGISTER_INDEX=> set_nr24(&mut self.apu.tone_channel, &self.apu.frame_sequencer, value),
            NR30_REGISTER_INDEX=> set_nr30(&mut self.apu.wave_channel, value),
            NR31_REGISTER_INDEX=> set_nr31(&mut self.apu.wave_channel, value),
            NR32_REGISTER_INDEX=> set_nr32(&mut self.apu.wave_channel, value),
            NR33_REGISTER_INDEX=> set_nr33(&mut self.apu.wave_channel, value),
            NR34_REGISTER_INDEX=> set_nr34(&mut self.apu.wave_channel, &self.apu.frame_sequencer, value),
            NR41_REGISTER_INDEX=> set_nr41(&mut self.apu.noise_channel, value),
            NR42_REGISTER_INDEX=> set_nr42(&mut self.apu.noise_channel, value),
            NR43_REGISTER_INDEX=> set_nr43(&mut self.apu.noise_channel, value),
            NR44_REGISTER_INDEX=> set_nr44(&mut self.apu.noise_channel, &self.apu.frame_sequencer, value),
            NR50_REGISTER_INDEX=> set_nr50(&mut self.apu, value),
            NR51_REGISTER_INDEX=> set_nr51(&mut self.apu, value),
            NR52_REGISTER_INDEX=> set_nr52(&mut self.apu, value),
            WAVE_RAM_START_INDEX..=WAVE_RAM_END_INDEX => set_wave_ram(&mut self.apu.wave_channel, address, value), 
            //PPU
            LCDC_REGISTER_INDEX=> self.ppu.set_lcdcontrol_register(value),
            STAT_REGISTER_INDEX=> self.ppu.set_stat_register(value),
            SCY_REGISTER_INDEX=> self.ppu.set_scy(value),
            SCX_REGISTER_INDEX=> self.ppu.set_scx(value),
            // LY is readonly
            LYC_REGISTER_INDEX=> self.ppu.set_lyc(value),
            DMA_REGISTER_INDEX=>self.oam_dma_controller.set_dma_register(value),
            BGP_REGISTER_INDEX=> self.ppu.set_bg_palette_register(value),
            OBP0_REGISTER_INDEX=> self.ppu.set_obp_palette_register(value, false),
            OBP1_REGISTER_INDEX=> self.ppu.set_obp_palette_register(value, true),
            WY_REGISTER_INDEX=> self.ppu.set_wy_register(value),
            WX_REGISTER_INDEX=> self.ppu.set_wx_register(value),
            JOYP_REGISTER_INDEX => self.joypad_handler.set_register(value),

            // CGB registers
            _ if self.mode == Mode::CGB => match address {
                VBK_REGISTER_INDEX =>self.ppu.vram.set_bank_reg(value),
                KEY0_REGISTER_INDEX => {
                    self.key0_register = value;
                    if !self.boot_finished{
                        self.ppu.cgb_enabled = self.key0_register & BIT_2_MASK == 0;
                    }
                }
                KEY1_REGISTER_INDEX =>{
                    self.speed_switch_register &= 0b1111_1110;    // clear bit 0
                    self.speed_switch_register |= value & 1;      // change state for bit 0
                }
                ORPI_REGISTER_INDEX => self.ppu.set_orpi(value),
                // VRAM DMA
                HDMA1_REGISTER_INDEX =>self.vram_dma_controller.set_source_high(value),
                HDMA2_REGISTER_INDEX =>self.vram_dma_controller.set_source_low(value),
                HDMA3_REGISTER_INDEX =>self.vram_dma_controller.set_dest_high(value),
                HDMA4_REGISTER_INDEX =>self.vram_dma_controller.set_dest_low(value),
                HDMA5_REGISTER_INDEX =>self.vram_dma_controller.set_mode_length(value),
                // COLOR Ram
                BGPI_REGISTER_INDEX =>self.ppu.set_bgpi(value),
                BGPD_REGISTER_INDEX =>self.ppu.set_bgpd(value),
                OBPI_REGISTER_INDEX =>self.ppu.set_obpi(value),
                OBPD_REGISTER_INDEX =>self.ppu.set_obpd(value),
                _=>{}
            }
            _=>{}
        }
    }
    
    pub fn new(apu:GbApu<AD>, gfx_device:GFX, joypad_provider:JP, mode:Mode)->Self{
        Self{
            apu,
            timer:GbTimer::default(),
            ppu:GbPpu::new(gfx_device, mode),
            oam_dma_controller: OamDmaController::new(),
            vram_dma_controller: VramDmaController::new(),
            interrupt_handler: InterruptsHandler::default(),
            joypad_handler: JoypadHandler::new(joypad_provider),
            speed_switch_register:0,
            speed_cycle_reminder:0,
            apu_cycles_counter:0,
            ppu_cycles:0,
            timer_cycles:0,
            ppu_event:None,
            timer_event_cycles: 0,
            apu_event_cycles: 0,
            boot_finished: false,
            key0_register: 0, 
            mode,
        }
    }

    pub fn cycle(&mut self, mut cycles:u32, double_speed_mode:bool, halt: bool, external_memory_bus:&mut ExternalMemoryBus)->Option<AccessBus>{
        // Timer is effected by double speed mode so handling it first
        self.timer_cycles += cycles;
        
        if self.timer_event_cycles <= self.timer_cycles{
            self.cycle_timer();
        }

        let access_bus = self.oam_dma_controller.cycle(cycles, external_memory_bus, &mut self.ppu);

        // APU, PPU and vram dma are not effected by the speed mode
        if double_speed_mode{
            cycles += self.speed_cycle_reminder as u32;
            self.speed_cycle_reminder = cycles as u8 & 1;   // Saves the LSB (the bit to indicate odd number)
            cycles >>= 1;                                   // divide by 2 (discard the LSB bit)
        }

        if !halt {
            // HDMA is disabled during halt mode
            self.vram_dma_controller.cycle(cycles, external_memory_bus, &mut self.ppu);
        }

        self.apu_cycles_counter += cycles;
        
        if !self.ppu_event.is_none(){
            self.ppu_cycles += cycles;
        }
        else{
            self.ppu_cycles = 0;
            // When screen is off constantly try and check for activation by cycling with 0 
            self.cycle_ppu();
        }

        if let Some(cycles) = self.ppu_event{
            if cycles <= self.ppu_cycles{
                self.cycle_ppu();
            }
        }
        if self.apu_event_cycles <= self.apu_cycles_counter{
            self.cycle_apu();
        }

        return access_bus;
    }

    pub fn set_boot_finished(&mut self){self.boot_finished = true}

    pub fn is_cgb_enabled(&self)->bool{self.ppu.cgb_enabled}

    fn cycle_ppu(&mut self){
        self.ppu_event = self.ppu.cycle(self.ppu_cycles, &mut self.interrupt_handler.interrupt_flag);
        self.ppu_cycles = 0;
    }

    fn cycle_apu(&mut self){
        #[cfg(feature = "apu")]{
            self.apu_event_cycles = self.apu.cycle(self.apu_cycles_counter);
        }
        
        self.apu_cycles_counter = 0;
    }

    fn cycle_timer(&mut self){
        self.timer_event_cycles = self.timer.cycle(self.timer_cycles, &mut self.interrupt_handler.interrupt_flag);
        self.timer_cycles = 0;
    }
}