use crate::{apu::{audio_device::AudioDevice, gb_apu::GbApu, set_nr11, set_nr12, set_nr13, volume_envelop::VolumeEnvlope}, timer::timer_register_updater::*, utils::{bit_masks::BIT_3_MASK, memory_registers::*}};
use crate::ppu::gb_ppu::GbPpu;
use crate::apu::*;
use crate::timer::gb_timer::GbTimer;
use super::{io_ports::IoPorts, memory::{Memory, UnprotectedMemory}, oam_dma_transferer::OamDmaTransferer};
use super::io_ports::*;

pub struct IoComps<AD:AudioDevice>{
    pub apu: GbApu<AD>,
    pub timer: GbTimer,
    pub ports:IoPorts,
}

impl<AD:AudioDevice> Memory for IoComps<AD>{
    fn read(&self, address:u16)->u8 {
        let mut value = self.ports.read(address);
        match address {
            DIV_REGISTER_INDEX=> value = get_div(&self.timer),
            TIMA_REGISTER_INDEX=> value = self.timer.tima_register,
            NR52_REGISTER_INDEX=> get_nr52(&self.apu, &mut value),
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
            _=>{}
        }

        self.ports.write(address, value);
    }
}

impl<AD:AudioDevice> IoComps<AD>{
    pub fn cycle(&mut self, cycles:u32){
        let mut if_register = self.ports.read_unprotected(IF_REGISTER_ADDRESS - 0xFF00);
        self.timer.cycle(&mut if_register, cycles as u8);
        self.apu.cycle(cycles as u8);
        self.ports.write_unprotected(IF_REGISTER_ADDRESS - 0xFF00, if_register);
    }
}