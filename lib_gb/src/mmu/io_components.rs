use crate::{apu::{audio_device::AudioDevice, gb_apu::GbApu, set_nr11, set_nr12, set_nr13}, ppu::ppu_register_updater::*, timer::timer_register_updater::*, utils::memory_registers::*};
use crate::ppu::gb_ppu::GbPpu;
use crate::apu::*;
use crate::timer::gb_timer::GbTimer;
use super::{access_bus::AccessBus, io_ports::IoPorts, memory::{Memory, UnprotectedMemory}, oam_dma_transferer::OamDmaTransferer, ram::Ram};
use super::io_ports::*;

pub struct IoComponents<AD:AudioDevice>{
    pub ram: Ram,
    pub apu: GbApu<AD>,
    pub timer: GbTimer,
    pub ppu:GbPpu,
    pub ports:IoPorts,
    pub dma:OamDmaTransferer,
    pub finished_boot:bool,
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


impl<AD:AudioDevice> Memory for IoComponents<AD>{
    fn read(&self, address:u16)->u8 {
        let mut value = self.ports.read(address);
        match address {
            //Timer
            DIV_REGISTER_INDEX=> value = get_div(&self.timer),
            TIMA_REGISTER_INDEX=> value = self.timer.tima_register,
            //APU
            NR52_REGISTER_INDEX=> get_nr52(&self.apu, &mut value),
            //PPU
            STAT_REGISTER_INDEX=> value = get_stat(&self.ppu),
            LY_REGISTER_INDEX=> value = get_ly(&self.ppu),
            _=>{}
        }

        value
    }

    fn write(&mut self, address:u16, value:u8) {
        match address{
            //timer
            DIV_REGISTER_INDEX=> reset_div(&mut self.timer),
            TIMA_REGISTER_INDEX=> set_tima(&mut self.timer, value),
            TMA_REGISTER_INDEX=> set_tma(&mut self.timer, value),
            TAC_REGISTER_INDEX=> set_tac(&mut self.timer, value),
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
            NR34_REGISTER_INDEX=> set_nr34(&mut self.apu.wave_channel, &self.apu.frame_sequencer, self.ports.read_unprotected(NR30_REGISTER_INDEX),value),
            NR41_REGISTER_INDEX=> set_nr41(&mut self.apu.noise_channel, value),
            NR42_REGISTER_INDEX=> set_nr42(&mut self.apu.noise_channel, value),
            NR43_REGISTER_INDEX=> set_nr43(&mut self.apu.noise_channel, value),
            NR44_REGISTER_INDEX=> set_nr44(&mut self.apu.noise_channel, &self.apu.frame_sequencer, value),
            NR50_REGISTER_INDEX=> set_nr50(&mut self.apu, value),
            NR51_REGISTER_INDEX=> set_nr51(&mut self.apu, value),
            NR52_REGISTER_INDEX=> set_nr52(&mut self.apu, &mut self.ports,value),
            //PPU
            LCDC_REGISTER_INDEX=> handle_lcdcontrol_register(value, &mut self.ppu),
            STAT_REGISTER_INDEX=> update_stat_register(value, &mut self.ppu),
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
            // TODO: handle gbc registers (expecailly ram and vram)
            _=>{}
        }

        self.ports.write(address, value);
    }
}

impl<AD:AudioDevice> IoComponents<AD>{
    pub fn new(apu:GbApu<AD>)->Self{
        Self{apu, ports:IoPorts::default(), timer:GbTimer::default(), ppu:GbPpu::default(), dma:OamDmaTransferer::default(),finished_boot:false, ram:Ram::default()}
    }

    pub fn cycle(&mut self, cycles:u32){
        let mut if_register = self.ports.read_unprotected(IF_REGISTER_ADDRESS - 0xFF00);
        self.timer.cycle(&mut if_register, cycles as u8);
        self.apu.cycle(cycles as u8);
        self.ppu.update_gb_screen(&mut if_register, cycles);
        self.ports.write_unprotected(IF_REGISTER_ADDRESS - 0xFF00, if_register);
    }
}