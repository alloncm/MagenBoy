use crate::{
    mmu::{gb_mmu::GbMmu, memory::UnprotectedMemory}, 
    utils::{bit_masks::*, memory_registers::{
        NR21_REGISTER_ADDRESS, NR24_REGISTER_ADDRESS, NR30_REGISTER_ADDRESS, NR41_REGISTER_ADDRESS, NR44_REGISTER_ADDRESS
    }}
};
use self::{audio_device::AudioDevice, channel::Channel, frame_sequencer::FrameSequencer, freq_sweep::FreqSweep, gb_apu::GbApu, noise_sample_producer::NoiseSampleProducer, sample_producer::SampleProducer, tone_sample_producer::ToneSampleProducer, tone_sweep_sample_producer::ToneSweepSampleProducer, volume_envelop::VolumeEnvlope, wave_sample_producer::WaveSampleProducer};

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
        prepare_wave_channel(&mut apu.wave_channel, memory, &apu.frame_sequencer);
        prepare_tone_sweep_channel(&mut apu.sweep_tone_channel, memory, &apu.frame_sequencer);
        prepare_noise_channel(&mut apu.noise_channel, memory, &apu.frame_sequencer);
        prepare_tone_channel(&mut apu.tone_channel, memory, &apu.frame_sequencer);
    }
}

fn prepare_tone_channel(channel:&mut Channel<ToneSampleProducer>, memory:&mut GbMmu,fs:&FrameSequencer){ 

    if memory.io_ports.get_ports_cycle_trigger()[0x16]{
        channel.sound_length = 64 - (memory.read_unprotected(NR21_REGISTER_ADDRESS) & 0b11_1111) as u16;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x17]{
        update_volume_envelope(&mut channel.volume, memory.read_unprotected(0xFF17), &mut channel.sample_producer.envelop);
        
        if !is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope){
            channel.enabled = false;
        }
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x18]{
        //discard lower bit
        channel.frequency >>= 8;
        channel.frequency <<= 8;
        channel.frequency |= memory.read_unprotected(0xFF18) as u16;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x19]{
        //log::warn!("vol: {} swp: {}", channel.volume, channel.sample_producer.envelop.increase_envelope);
        let nr24  = memory.read_unprotected(NR24_REGISTER_ADDRESS);
        channel.frequency |= (nr24 as u16 & 0b111) << 8;
        let dac_enabled = is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope);
        update_channel_conrol_register(channel, dac_enabled, nr24, 64, fs);
    }
}

fn prepare_noise_channel(channel:&mut Channel<NoiseSampleProducer>, memory:&mut GbMmu,fs:&FrameSequencer){

    if memory.io_ports.get_ports_cycle_trigger()[0x20]{
        let length_data = memory.read_unprotected(NR41_REGISTER_ADDRESS) & 0b11_1111;
        channel.sound_length = 64 - length_data as u16
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x21]{
        update_volume_envelope(&mut channel.volume, memory.read_unprotected(0xFF21), &mut channel.sample_producer.envelop);
        if !is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope){
            channel.enabled = false;
        }
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x23]{
        
        let nr44 = memory.read_unprotected(NR44_REGISTER_ADDRESS);
        let dac_enabled = is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope);
        update_channel_conrol_register(channel, dac_enabled, nr44, 64, fs);
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

fn prepare_wave_channel(channel:&mut Channel<WaveSampleProducer>, memory:&mut GbMmu,fs:&FrameSequencer){

    if memory.io_ports.get_ports_cycle_trigger()[0x1A]{
        if (memory.read_unprotected(NR30_REGISTER_ADDRESS) & BIT_7_MASK) == 0{
            channel.enabled = false;
        }
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x1B]{
        channel.sound_length = 256 - (memory.read_unprotected(0xFF1B) as u16);
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x1C]{
        //I want bits 5-6
        channel.sample_producer.volume = (memory.read_unprotected(0xFF1C)>>5) & 0b011;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x1D]{
        
        channel.frequency = memory.read_unprotected(0xFF1D) as u16;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x1E]{
        let nr34 = memory.read_unprotected(0xFF1E);
        channel.frequency |= ((nr34 & 0b111) as u16) << 8;

        // According to the docs the frequency is 65536/(2048-x) Hz
        // After some calculations if we are running in 0x400000 Hz this should be the 
        // amount of cycles we should trigger a new sample
        // 
        // Rate is for how many cycles I should trigger.
        // So I did the frequency of the cycles divided by the frequency of this channel
        // which is 0x400000 / 65536 (2048 - x) = 64(2048 - x)
        //let timer_cycles_to_tick = (2048 - channel.frequency).wrapping_mul(64);

        let dac_enabled = (memory.read_unprotected(NR30_REGISTER_ADDRESS) & BIT_7_MASK) != 0;
        update_channel_conrol_register(channel, dac_enabled, nr34, 256, fs);
    }

    for i in 0..=0xF{
        channel.sample_producer.wave_samples[i] = memory.read_unprotected(0xFF30 + i as u16);
    }
}

fn prepare_tone_sweep_channel(channel:&mut Channel<ToneSweepSampleProducer>, memory:&mut GbMmu, fs:&FrameSequencer){
    let nr10 = memory.read_unprotected(0xFF10);
    let nr11 = memory.read_unprotected(0xFF11);
    let nr12 = memory.read_unprotected(0xFF12);
    let nr13 = memory.read_unprotected(0xFF13);
    let nr14 = memory.read_unprotected(0xFF14);

    if memory.io_ports.get_ports_cycle_trigger()[0x10]{
        //sweep
        channel.sample_producer.sweep.sweep_decrease = (nr10 & 0b1000) != 0;
        channel.sample_producer.sweep.sweep_shift = nr10 & 0b111;
        channel.sample_producer.sweep.sweep_period = (nr10 & 0b111_0000) >> 4;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x11]{
        channel.sample_producer.wave_duty = (nr11 & 0b1100_0000) >> 6;
        channel.sound_length = 64 - (nr11 & 0b11_1111) as u16
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x12]{
        update_volume_envelope(&mut channel.volume, nr12, &mut channel.sample_producer.envelop);
        
        if !is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope){
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
        //discard upper bit
        channel.frequency <<= 8;
        channel.frequency >>= 8;
        channel.frequency |= ((nr14 & 0b111) as u16) << 8;
        
        let dac_enabled = is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope);
        update_channel_conrol_register(channel, dac_enabled, nr14, 64, fs);

        if nr14 & BIT_7_MASK != 0{
            //volume
            channel.sample_producer.envelop.envelop_duration_counter = 0;
            
            //sweep
            channel.sample_producer.sweep.channel_trigger(channel.frequency);
            if channel.sample_producer.sweep.sweep_shift > 0{
                
                let freq = channel.sample_producer.sweep.calculate_new_frequency();
                channel.enabled = !FreqSweep::check_overflow(freq);
            }
        }
    }
}

fn update_channel_conrol_register<T:SampleProducer>(channel:&mut Channel<T>, dac_enabled:bool, control_register:u8, 
    max_sound_length:u16, fs:&FrameSequencer){

    let previous_length_enable = channel.length_enable;

    channel.length_enable = (control_register & BIT_6_MASK) !=0;

    //the folowing behavior vary between gb and gbc
    let possible_extra_length_clocking  = !previous_length_enable && channel.length_enable && channel.sound_length != 0;

    if possible_extra_length_clocking{
        if !fs.should_next_step_clock_length(){
            channel.update_length_register();
        }
    }

    if (control_register & BIT_7_MASK) != 0{
        if dac_enabled{
            channel.enabled = true;
        }

        if channel.sound_length == 0{
            channel.sound_length = max_sound_length;

            if channel.length_enable && !fs.should_next_step_clock_length(){
                channel.update_length_register();
            }
        }

        channel.timer.update_cycles_to_tick(T::get_updated_frequency_ticks(channel.frequency));
    }
}

fn update_volume_envelope(volume: &mut u8, register:u8, envelop:&mut VolumeEnvlope){
    *volume = (register & 0b1111_0000) >> 4;
    envelop.number_of_envelope_sweep = register & 0b111;
    envelop.increase_envelope = (register & BIT_3_MASK) != 0;
}

fn is_dac_enabled(volume:u8, envelop_increase:bool)->bool{
    volume != 0 || envelop_increase
}