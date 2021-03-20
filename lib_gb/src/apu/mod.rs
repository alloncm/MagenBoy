use crate::{mmu::{gb_mmu::GbMmu, memory::UnprotectedMemory}, utils::{bit_masks::*, memory_registers::{NR21_REGISTER_ADDRESS, NR24_REGISTER_ADDRESS, NR41_REGISTER_ADDRESS, NR44_REGISTER_ADDRESS}}};
use self::{audio_device::AudioDevice, gb_apu::GbApu};

pub mod gb_apu;
pub mod channel;
pub mod sample_producer;
pub mod wave_sample_producer;
pub mod audio_device;
pub mod timer;
pub mod frame_sequencer;
pub mod sound_terminal;
pub mod tone_sweep_sample_producer;
pub mod freq_sweep;
pub mod volume_envelop;
pub mod tone_sample_producer;
pub mod noise_sample_producer;
mod sound_utils;

pub fn update_apu_registers<AD:AudioDevice>(memory:&mut GbMmu, apu:&mut GbApu<AD>){
    prepare_control_registers(apu, memory);

    if apu.enabled{
        prepare_wave_channel(apu, memory);
        prepare_tone_sweep_channel(apu, memory);
        prepare_noise_channel(apu, memory);
        prepare_tone_channel(apu, memory);
    }
}

fn prepare_tone_channel<AD:AudioDevice>(apu: &mut GbApu<AD>, memory:&mut impl UnprotectedMemory){
    let nr24  = memory.read_unprotected(NR24_REGISTER_ADDRESS);
    if nr24 & BIT_7_MASK != 0{

        let nr21 = memory.read_unprotected(NR21_REGISTER_ADDRESS);
        apu.tone_channel.sound_length = nr21 & 0b11_1111;
        apu.tone_channel.length_enable = nr24 & BIT_6_MASK != 0;
        apu.tone_channel.sample_producer.wave_duty = (nr21 & 0b1100_0000) >> 6;
        memory.write_unprotected(NR24_REGISTER_ADDRESS, nr24 & 0b0111_1111);
    }
}

fn prepare_noise_channel<AD:AudioDevice>(apu: &mut GbApu<AD>, memory:&mut impl UnprotectedMemory){
    let nr44 = memory.read_unprotected(NR44_REGISTER_ADDRESS);
    if nr44 & BIT_7_MASK != 0{
        let nr41 = memory.read_unprotected(NR41_REGISTER_ADDRESS);
        apu.noise_channel.sound_length = nr41 & 0b11_1111;
        apu.noise_channel.length_enable = nr44 & BIT_6_MASK != 0;
        memory.write_unprotected(NR44_REGISTER_ADDRESS, nr44 & 0b0111_1111);
    }
}

fn prepare_control_registers<AD:AudioDevice>(apu:&mut GbApu<AD>, memory:&impl UnprotectedMemory){
    let channel_control = memory.read_unprotected(0xFF24);
    apu.terminal1.enabled = channel_control & BIT_3_MASK != 0;
    apu.terminal2.enabled = channel_control & BIT_7_MASK != 0;
    //todo: add volume
    apu.terminal1.volume = 7;
    apu.terminal2.volume = 7;

    let channels_output_terminals = memory.read_unprotected(0xFF25);

    for i in 0..4{
        apu.terminal1.channels[i as usize] = channels_output_terminals & (1 << i) != 0;
    }
    for i in 0..4{
        apu.terminal2.channels[i as usize] = channels_output_terminals & (0b10000 << i) != 0;
    }

    let master_sound = memory.read_unprotected(0xFF26);
    apu.enabled = master_sound & BIT_7_MASK != 0;
}

fn prepare_wave_channel<AD:AudioDevice>(apu: &mut GbApu<AD>, memory:&impl UnprotectedMemory){
    apu.wave_channel.sound_length = memory.read_unprotected(0xFF1B);
    apu.wave_channel.enabled = memory.read_unprotected(0xFF1A) & BIT_7_MASK != 0;
    //I want bits 5-6
    apu.wave_channel.sample_producer.volume = (memory.read_unprotected(0xFF1C)>>5) & 0b011;
    let mut freq = memory.read_unprotected(0xFF1D) as u16;
    let nr34 = memory.read_unprotected(0xFF1E);
    freq |= ((nr34 & 0b111) as u16) << 8;
    apu.wave_channel.frequency = freq;

    // According to the docs the frequency is 65536/(2048-x) Hz
    // After some calculations if we are running in 0x400000 Hz this should be the 
    // amount of cycles we should trigger a new sample
    // 
    // Rate is for how many cycles I should trigger.
    // So I did the frequency of the cycles divided by the frequency of this channel
    // which is 0x400000 / 65536 (2048 - x) = 64(2048 - x)
    apu.wave_channel.timer.update_cycles_to_tick((2048 - freq).wrapping_mul(64));
    
    apu.wave_channel.length_enable = nr34 & BIT_6_MASK != 0;

    for i in 0..=0xF{
        apu.wave_channel.sample_producer.wave_samples[i] = memory.read_unprotected(0xFF30 + i as u16);
    }
}

fn prepare_tone_sweep_channel<AD:AudioDevice>(apu:&mut GbApu<AD>, memory:&mut GbMmu){
    let channel = &mut apu.sweep_tone_channel;

    let nr10 = memory.read_unprotected(0xFF10);
    let nr11 = memory.read_unprotected(0xFF11);
    let nr12 = memory.read_unprotected(0xFF12);
    let nr13 = memory.read_unprotected(0xFF13);
    let nr14 = memory.read_unprotected(0xFF14);


    if memory.io_ports.get_ports_cycle_trigger()[0x10]{
        //sweep
        channel.sample_producer.sweep.sweep_decrease = (nr10 & 0b1000) != 0;
        channel.sample_producer.sweep.sweep_shift = nr10 & 0b111;
        channel.sample_producer.sweep.time_sweep = (nr10 & 0b111_0000) >> 4;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x11]{
        channel.sample_producer.wave_duty = (nr11 & 0b1100_0000) >> 6;
        let sound_length = nr11 & 0b11_1111;
        if sound_length != 0{
            channel.sound_length = sound_length;
        }
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x12]{
        
        channel.volume = (nr12 & 0b1111_0000) >> 4;
        channel.sample_producer.envelop.number_of_envelope_sweep = nr12 & 0b111;
        channel.sample_producer.envelop.increase_envelope = (nr12 & BIT_3_MASK) != 0;
        if channel.volume == 0 && channel.sample_producer.envelop.increase_envelope{
            channel.enabled = false;
        }
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x13]{
        //discard lower bit
        channel.frequency >>= 8;
        channel.frequency <<= 8;
        channel.frequency |= nr13 as u16;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x14]{
        channel.length_enable = (nr14 & BIT_6_MASK) != 0;

        //discard upper bit
        channel.frequency <<= 8;
        channel.frequency >>= 8;
        channel.frequency |= ((nr14 & 0b111) as u16) << 8;

        if nr14 & 0b1000_0000 != 0{
            channel.enabled = true;
            if channel.sound_length == 0{
                channel.sound_length = 64;  
            }

            // See the wave for the calculation this channle freq is 131072/(2048-x) Hz
            channel.timer.update_cycles_to_tick((2048 - channel.frequency).wrapping_mul(4));

            //volume
            channel.sample_producer.envelop.envelop_duration_counter = 0;
            if channel.volume == 0 && channel.sample_producer.envelop.increase_envelope{
                channel.enabled = false;
                log::warn!("sweep tone is disabled");
            }
            
            //sweep
            channel.sample_producer.sweep.shadow_frequency = channel.frequency;

        }
    }
}

