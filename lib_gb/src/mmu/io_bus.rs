use crate::{
    apu::{*,audio_device::AudioDevice, gb_apu::GbApu}, 
    ppu::{gb_ppu::GbPpu, ppu_register_updater::*, gfx_device::GfxDevice},
    timer::{timer_register_updater::*, gb_timer::GbTimer}, 
    keypad::{joypad_provider::JoypadProvider, joypad_handler::JoypadHandler}
};
use super::{
    interrupts_handler::*, access_bus::AccessBus, memory::*, 
    oam_dma_transfer::OamDmaTransfer, ram::Ram, io_ports::*
};

pub const IO_PORTS_SIZE:usize = 0x80;
const WAVE_RAM_START_INDEX:u16 = 0x30;
const WAVE_RAM_END_INDEX:u16 = 0x3F;

pub struct IoBus<AD:AudioDevice, GFX:GfxDevice, JP:JoypadProvider>{
    pub ram: Ram,
    pub apu: GbApu<AD>,
    pub timer: GbTimer,
    pub ppu:GbPpu<GFX>,
    pub dma:OamDmaTransfer,
    pub interrupt_handler:InterruptsHandler,
    pub joypad_handler: JoypadHandler<JP>,
    pub finished_boot:bool,

    apu_cycles_counter:u32,
    ppu_cycles:u32,
    timer_cycles:u32,

    timer_event_cycles:u32,
    apu_event_cycles:u32,

    // Since the PPU is the only that can be completely off and not operate at all Im using an option
    // where None indicates the PPU is off
    ppu_event:Option<u32>,
}

impl<AD:AudioDevice, GFX:GfxDevice, JP:JoypadProvider> Memory for IoBus<AD, GFX, JP>{
    fn read(&mut self, address:u16)->u8 {

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
            STAT_REGISTER_INDEX=> get_stat(&self.ppu),
            SCY_REGISTER_INDEX=> self.ppu.bg_pos.y,
            SCX_REGISTER_INDEX=> self.ppu.bg_pos.x,
            LY_REGISTER_INDEX=> self.ppu.ly_register,
            LYC_REGISTER_INDEX=> self.ppu.lyc_register,
            DMA_REGISTER_INDEX=> (self.dma.soure_address >> 8) as u8,
            BGP_REGISTER_INDEX=> self.ppu.bg_palette_register,
            OBP0_REGISTER_INDEX=> self.ppu.obj_pallete_0_register,
            OBP1_REGISTER_INDEX=> self.ppu.obj_pallete_1_register,
            WY_REGISTER_INDEX => self.ppu.window_pos.y,
            WX_REGISTER_INDEX=> get_wx_register(&self.ppu),
            //BOOT
            BOOT_REGISTER_INDEX=> self.finished_boot as u8,
            //Joypad
            JOYP_REGISTER_INDEX => self.joypad_handler.register,
            _=>0xFF
        };
    }

    fn write(&mut self, address:u16, value:u8) {
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
            LCDC_REGISTER_INDEX=> handle_lcdcontrol_register(value, &mut self.ppu),
            STAT_REGISTER_INDEX=> update_stat_register(value, &mut self.ppu),
            SCY_REGISTER_INDEX=> set_scy(&mut self.ppu, value),
            SCX_REGISTER_INDEX=> set_scx(&mut self.ppu, value),
            // LY is readonly
            LYC_REGISTER_INDEX=> set_lyc(&mut self.ppu, value),
            DMA_REGISTER_INDEX=>{
                let address = (value as u16) << 8;
                self.dma.soure_address = address;
                self.dma.enable = match value{
                    0..=0x7F=>Some(AccessBus::External),
                    0x80..=0x9F=>Some(AccessBus::Video),
                    0xA0..=0xFF=>Some(AccessBus::External)
                }
            }
            BGP_REGISTER_INDEX=> handle_bg_pallet_register(value,&mut self.ppu.bg_color_mapping, &mut self.ppu.bg_palette_register),
            OBP0_REGISTER_INDEX=> handle_obp_pallet_register(value,&mut self.ppu.obj_color_mapping0, &mut self.ppu.obj_pallete_0_register),
            OBP1_REGISTER_INDEX=> handle_obp_pallet_register(value,&mut self.ppu.obj_color_mapping1, &mut self.ppu.obj_pallete_1_register),
            WY_REGISTER_INDEX=> handle_wy_register(value, &mut self.ppu),
            WX_REGISTER_INDEX=> handle_wx_register(value, &mut self.ppu),
            BOOT_REGISTER_INDEX=> self.finished_boot = value != 0,
            JOYP_REGISTER_INDEX => self.joypad_handler.set_register(value),
            // TODO: handle gbc registers (expecailly ram and vram)
            _=>{}
        }
    }
}

impl<AD:AudioDevice, GFX:GfxDevice, JP:JoypadProvider> IoBus<AD, GFX, JP>{
    pub fn new(apu:GbApu<AD>, gfx_device:GFX, joypad_provider:JP)->Self{
        Self{
            apu,
            timer:GbTimer::default(),
            ppu:GbPpu::new(gfx_device),
            dma:OamDmaTransfer::default(),
            interrupt_handler: InterruptsHandler::default(),
            joypad_handler: JoypadHandler::new(joypad_provider),
            finished_boot:false,
            ram:Ram::default(),
            apu_cycles_counter:0,
            ppu_cycles:0,
            timer_cycles:0,
            ppu_event:None,
            timer_event_cycles: 0,
            apu_event_cycles: 0,
        }
    }

    pub fn cycle(&mut self, cycles:u32){
        self.apu_cycles_counter += cycles;
        self.timer_cycles += cycles;
        
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
        if self.timer_event_cycles <= self.timer_cycles{
            self.cycle_timer();
        }
        if self.apu_event_cycles <= self.apu_cycles_counter{
            self.cycle_apu();
        }
    }

    fn cycle_ppu(&mut self){
        self.ppu_event = self.ppu.cycle(self.ppu_cycles, &mut self.interrupt_handler.interrupt_flag);
        self.ppu_cycles = 0;
    }
    fn cycle_apu(&mut self){
        self.apu_event_cycles = self.apu.cycle(self.apu_cycles_counter);
        self.apu_cycles_counter = 0;
    }
    fn cycle_timer(&mut self){
        self.timer_event_cycles = self.timer.cycle(self.timer_cycles, &mut self.interrupt_handler.interrupt_flag);
        self.timer_cycles = 0;
    }
}