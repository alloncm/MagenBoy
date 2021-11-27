use crate::{apu::{*,audio_device::AudioDevice, gb_apu::GbApu}, 
    ppu::{gb_ppu::GbPpu, ppu_register_updater::*, gfx_device::GfxDevice},
    timer::timer_register_updater::*, 
    utils::memory_registers::*
};
use crate::timer::gb_timer::GbTimer;
use super::{access_bus::AccessBus, memory::*, oam_dma_transfer::OamDmaTransfer, ram::Ram, scheduler::ScheduledEvent};
use super::io_ports::*;

pub const IO_PORTS_SIZE:usize = 0x80;
const WAVE_RAM_START_INDEX:u16 = 0x30;
const WAVE_RAM_END_INDEX:u16 = 0x3F;

pub struct IoComponents<AD:AudioDevice, GFX:GfxDevice>{
    pub ram: Ram,
    pub apu: GbApu<AD>,
    pub timer: GbTimer,
    pub ppu:GbPpu<GFX>,
    pub dma:OamDmaTransfer,
    pub finished_boot:bool,

    ports:[u8;IO_PORTS_SIZE],
    apu_cycles:u32,
    ppu_cycles:u32,
    timer_cycles:u32,

    timer_event:Option<ScheduledEvent>,
    ppu_event:Option<ScheduledEvent>,
    apu_event:Option<ScheduledEvent>,
}

io_port_index!(LCDC_REGISTER_INDEX, LCDC_REGISTER_ADDRESS);
io_port_index!(STAT_REGISTER_INDEX, STAT_REGISTER_ADDRESS);
io_port_index!(SCY_REGISTER_INDEX, SCY_REGISTER_ADDRESS);
io_port_index!(SCX_REGISTER_INDEX, SCX_REGISTER_ADDRESS);
io_port_index!(LY_REGISTER_INDEX, LY_REGISTER_ADDRESS);
io_port_index!(LYC_REGISTER_INDEX, LYC_REGISTER_ADDRESS);
io_port_index!(DMA_REGISTER_INDEX, DMA_REGISTER_ADDRESS);
io_port_index!(WY_REGISTER_INDEX, WY_REGISTER_ADDRESS);
io_port_index!(WX_REGISTER_INDEX, WX_REGISTER_ADDRESS);
io_port_index!(BOOT_REGISTER_INDEX, BOOT_REGISTER_ADDRESS);
io_port_index!(BGP_REGISTER_INDEX, BGP_REGISTER_ADDRESS);
io_port_index!(OBP0_REGISTER_INDEX, OBP0_REGISTER_ADDRESS);
io_port_index!(OBP1_REGISTER_INDEX, OBP1_REGISTER_ADDRESS);
io_port_index!(IF_REGISTER_INDEX, IF_REGISTER_ADDRESS);

impl<AD:AudioDevice, GFX:GfxDevice> Memory for IoComponents<AD, GFX>{
    fn read(&mut self, address:u16)->u8 {

        match address{
            TAC_REGISTER_INDEX | DIV_REGISTER_INDEX | TIMA_REGISTER_INDEX=> self.cycle_timer(),
            NR10_REGISTER_INDEX..=WAVE_RAM_END_INDEX => self.cycle_apu(),
            LCDC_REGISTER_INDEX..=WX_REGISTER_INDEX => self.cycle_ppu(),
            _=>{}
        }

        let mut value = self.ports[address as usize];
        return match address {
            //Timer
            TAC_REGISTER_INDEX=> value & 0b111,
            DIV_REGISTER_INDEX=> get_div(&self.timer),
            TIMA_REGISTER_INDEX=> self.timer.tima_register,
            //APU
            NR10_REGISTER_INDEX=>value | 0b1000_0000,
            NR11_REGISTER_INDEX=> value | 0b0011_1111,
            NR13_REGISTER_INDEX=> 0xFF,
            NR14_REGISTER_INDEX=> value | 0b1011_1111,
            0x15 => 0xFF, //Not used
            NR21_REGISTER_INDEX=> value | 0b0011_1111,
            NR23_REGISTER_INDEX=> 0xFF,
            NR24_REGISTER_INDEX=> value | 0b1011_1111,
            NR30_REGISTER_INDEX=> value | 0b0111_1111,
            NR31_REGISTER_INDEX=> value | 0xFF,
            NR32_REGISTER_INDEX=> value | 0b1001_1111,
            NR33_REGISTER_INDEX=> value | 0xFF,
            NR34_REGISTER_INDEX=> value | 0b1011_1111,
            0x1F => 0xFF, //Not used
            NR41_REGISTER_INDEX=> 0xFF,
            NR44_REGISTER_INDEX=> value | 0b1011_1111,
            NR52_REGISTER_INDEX=> {
                get_nr52(&self.apu, &mut value);
                value
            }
            0x27..=0x2F => 0xFF, //Not used
            WAVE_RAM_START_INDEX..=WAVE_RAM_END_INDEX => get_wave_ram(&self.apu.wave_channel, address),
            //PPU
            STAT_REGISTER_INDEX=> get_stat(&self.ppu),
            LY_REGISTER_INDEX=> get_ly(&self.ppu),
            //Joypad
            JOYP_REGISTER_INDEX => {
                let joypad_value = self.ports[JOYP_REGISTER_INDEX as usize];
                (joypad_value & 0xF) | (value & 0xF0)
            }
            _=>value
        };
    }

    fn write(&mut self, address:u16, mut value:u8) {
        match address{
            DIV_REGISTER_INDEX | TIMA_REGISTER_INDEX | TMA_REGISTER_INDEX | TAC_REGISTER_INDEX => self.cycle_timer(),
            NR10_REGISTER_INDEX..=WAVE_RAM_END_INDEX => self.cycle_ppu(),
            LCDC_REGISTER_INDEX..=WX_REGISTER_INDEX => self.cycle_ppu(),
            _=>{}
        }
        match address{
            //timer
            DIV_REGISTER_INDEX=> {
                reset_div(&mut self.timer);
                value = 0;
            }
            TIMA_REGISTER_INDEX=> set_tima(&mut self.timer, value),
            TMA_REGISTER_INDEX=> set_tma(&mut self.timer, value),
            TAC_REGISTER_INDEX=> {
                set_tac(&mut self.timer, value);
                value &= 0b111;
            }
            //APU
            NR10_REGISTER_INDEX=> set_nr10(&mut self.apu.sweep_tone_channel, value),
            NR11_REGISTER_INDEX=> set_nr11(&mut self.apu.sweep_tone_channel, value),
            NR12_REGISTER_INDEX=> set_nr12(&mut self.apu.sweep_tone_channel, value),
            NR13_REGISTER_INDEX=> set_nr13(&mut self.apu.sweep_tone_channel, value),
            NR14_REGISTER_INDEX=> set_nr14(&mut self.apu.sweep_tone_channel, &self.apu.frame_sequencer, value),
            NR21_REGISTER_INDEX=> set_nr11(&mut self.apu.tone_channel, value),
            NR22_REGISTER_INDEX=> set_nr12(&mut self.apu.tone_channel, value),
            NR23_REGISTER_INDEX=> set_nr13(&mut self.apu.tone_channel, value),
            NR24_REGISTER_INDEX=> set_nr24(&mut self.apu.tone_channel, &self.apu.frame_sequencer, value),
            NR30_REGISTER_INDEX=> set_nr30(&mut self.apu.wave_channel, value),
            NR31_REGISTER_INDEX=> set_nr31(&mut self.apu.wave_channel, value),
            NR32_REGISTER_INDEX=> set_nr32(&mut self.apu.wave_channel, value),
            NR33_REGISTER_INDEX=> set_nr33(&mut self.apu.wave_channel, value),
            NR34_REGISTER_INDEX=> set_nr34(&mut self.apu.wave_channel, &self.apu.frame_sequencer, self.ports[NR30_REGISTER_INDEX as usize],value),
            NR41_REGISTER_INDEX=> set_nr41(&mut self.apu.noise_channel, value),
            NR42_REGISTER_INDEX=> set_nr42(&mut self.apu.noise_channel, value),
            NR43_REGISTER_INDEX=> set_nr43(&mut self.apu.noise_channel, value),
            NR44_REGISTER_INDEX=> set_nr44(&mut self.apu.noise_channel, &self.apu.frame_sequencer, value),
            NR50_REGISTER_INDEX=> set_nr50(&mut self.apu, value),
            NR51_REGISTER_INDEX=> set_nr51(&mut self.apu, value),
            NR52_REGISTER_INDEX=> set_nr52(&mut self.apu, &mut self.ports,value),
            WAVE_RAM_START_INDEX..=WAVE_RAM_END_INDEX => set_wave_ram(&mut self.apu.wave_channel, address, value), 
            //PPU
            LCDC_REGISTER_INDEX=> handle_lcdcontrol_register(value, &mut self.ppu),
            STAT_REGISTER_INDEX=> {
                update_stat_register(value, &mut self.ppu);
                value = (value >> 2) << 2;
            },
            SCY_REGISTER_INDEX=> set_scy(&mut self.ppu, value),
            SCX_REGISTER_INDEX=> set_scx(&mut self.ppu, value),
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
            BGP_REGISTER_INDEX=> handle_bg_pallet_register(value,&mut self.ppu.bg_color_mapping),
            OBP0_REGISTER_INDEX=> handle_obp_pallet_register(value,&mut self.ppu.obj_color_mapping0),
            OBP1_REGISTER_INDEX=> handle_obp_pallet_register(value,&mut self.ppu.obj_color_mapping1),
            WY_REGISTER_INDEX=> handle_wy_register(value, &mut self.ppu),
            WX_REGISTER_INDEX=> handle_wx_register(value, &mut self.ppu),
            BOOT_REGISTER_INDEX=> self.finished_boot = value != 0,
            JOYP_REGISTER_INDEX => {
                let joypad_value = self.ports[JOYP_REGISTER_INDEX as usize];
                value = (joypad_value & 0xF) | (value & 0xF0);
            }
            // TODO: handle gbc registers (expecailly ram and vram)
            _=>{}
        }

        self.ports[address as usize] = value;
    }
}

impl<AD:AudioDevice, GFX:GfxDevice> UnprotectedMemory for IoComponents<AD, GFX>{
    fn read_unprotected(&self, address:u16)->u8 {
        self.ports[address as usize]
    }

    fn write_unprotected(&mut self, address:u16, value:u8) {
        self.ports[address as usize] = value;
    }
}

impl<AD:AudioDevice, GFX:GfxDevice> IoComponents<AD, GFX>{
    pub fn new(apu:GbApu<AD>, gfx_device:GFX)->Self{
        Self{apu,
            ports:[0;IO_PORTS_SIZE],
            timer:GbTimer::default(),
            ppu:GbPpu::new(gfx_device), 
            dma:OamDmaTransfer::default(),
            finished_boot:false, 
            ram:Ram::default(),
            apu_cycles:0,
            ppu_cycles:0,
            timer_cycles:0,
            ppu_event:None,
            timer_event: None,
            apu_event:Some(ScheduledEvent{cycles:1, event_type:super::scheduler::ScheduledEventType::Ppu}),
        }
    }

    pub fn cycle(&mut self, cycles:u32){
        self.apu_cycles += cycles;
        self.ppu_cycles += cycles;
        self.timer_cycles += cycles;

        if let Some(event) = &self.ppu_event{
            if event.cycles <= self.ppu_cycles{
                self.cycle_ppu();
            }
        }
        if let Some(event) = &self.timer_event{
            if event.cycles <= self.timer_cycles{
                self.cycle_timer();
            }
        }
        if let Some(event) = &self.apu_event{
            if event.cycles <= self.apu_cycles{
                self.cycle_apu();
            }
        }
    }

    fn cycle_ppu(&mut self){
        let mut if_register = self.ports[IF_REGISTER_INDEX as usize];
        self.ppu_event = self.ppu.cycle(self.ppu_cycles, &mut if_register);
        self.ports[IF_REGISTER_INDEX as usize] = if_register;
        self.ppu_cycles = 0;
    }
    fn cycle_apu(&mut self){
        self.apu_event = self.apu.cycle(self.apu_cycles);
        self.apu_cycles = 0;
    }
    fn cycle_timer(&mut self){
        let mut if_register = self.ports[IF_REGISTER_INDEX as usize];
        self.timer_event = self.timer.cycle(self.timer_cycles, &mut if_register);
        self.ports[IF_REGISTER_INDEX as usize] = if_register;
        self.timer_cycles = 0;
    }
}