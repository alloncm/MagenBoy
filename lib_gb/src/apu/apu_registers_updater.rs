use crate::{
    mmu::{gb_mmu::GbMmu, memory::UnprotectedMemory},
    utils::{
        bit_masks::*, 
        memory_registers::{NR21_REGISTER_ADDRESS, NR24_REGISTER_ADDRESS, NR30_REGISTER_ADDRESS, NR41_REGISTER_ADDRESS, NR43_REGISTER_ADDRESS, NR44_REGISTER_ADDRESS}
    }
};

use super::{
    audio_device::AudioDevice, 
    channel::Channel, 
    frame_sequencer::FrameSequencer, 
    freq_sweep::FreqSweep, 
    gb_apu::GbApu, 
    noise_sample_producer::NoiseSampleProducer, 
    sample_producer::SampleProducer, 
    tone_sample_producer::ToneSampleProducer, 
    tone_sweep_sample_producer::ToneSweepSampleProducer, 
    volume_envelop::VolumeEnvlope, 
    wave_sample_producer::WaveSampleProducer
};


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
        let nr24  = memory.read_unprotected(NR24_REGISTER_ADDRESS);
        //discrad upper bit
        channel.frequency <<= 8;
        channel.frequency >>= 8;
        channel.frequency |= (nr24 as u16 & 0b111) << 8;
        let dac_enabled = is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope);
        update_channel_conrol_register(channel, dac_enabled, nr24, 64, fs);
        if nr24 & BIT_7_MASK != 0{
            //volume
            channel.sample_producer.envelop.envelop_duration_counter = channel.sample_producer.envelop.number_of_envelope_sweep;
        }
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
    if memory.io_ports.get_ports_cycle_trigger()[0x22]{
        let nr43 = memory.read_unprotected(NR43_REGISTER_ADDRESS);
        channel.sample_producer.bits_to_shift_divisor = (nr43 & 0b1111_0000) >> 4;
        channel.sample_producer.width_mode = (nr43 & BIT_3_MASK) != 0;
        channel.sample_producer.divisor_code = nr43 & 0b111;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x23]{
        let nr44 = memory.read_unprotected(NR44_REGISTER_ADDRESS);
        let dac_enabled = is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope);
        update_channel_conrol_register(channel, dac_enabled, nr44, 64, fs);
        if (nr44 & BIT_7_MASK) != 0{
            //On trigger all the LFSR bits are set (lfsr is 15 bit register)
            channel.sample_producer.lfsr = 0x7FFF;
        }
    }
}

fn prepare_control_registers<AD:AudioDevice>(apu:&mut GbApu<AD>, memory:&impl UnprotectedMemory){
    let channel_control = memory.read_unprotected(0xFF24);
    apu.right_terminal.enabled = channel_control & BIT_3_MASK != 0;
    apu.left_terminal.enabled = channel_control & BIT_7_MASK != 0;
    
    apu.right_terminal.volume = channel_control & 0b111;
    apu.left_terminal.volume = (channel_control & 0b111_0000) >> 4;

    let channels_output_terminals = memory.read_unprotected(0xFF25);

    for i in 0..4{
        apu.right_terminal.channels[i as usize] = channels_output_terminals & (1 << i) != 0;
    }
    for i in 0..4{
        apu.left_terminal.channels[i as usize] = channels_output_terminals & (0b10000 << i) != 0;
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
        channel.sample_producer.volume = (memory.read_unprotected(0xFF1C)>>5) & 0b11;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x1D]{
        
        channel.frequency = memory.read_unprotected(0xFF1D) as u16;
    }
    if memory.io_ports.get_ports_cycle_trigger()[0x1E]{
        let nr34 = memory.read_unprotected(0xFF1E);

        //clear the upper 8 bits
        channel.frequency &= 0xFF;
        channel.frequency |= ((nr34 & 0b111) as u16) << 8;

        let dac_enabled = (memory.read_unprotected(NR30_REGISTER_ADDRESS) & BIT_7_MASK) != 0;
        update_channel_conrol_register(channel, dac_enabled, nr34, 256, fs);

        if nr34 & BIT_7_MASK != 0{
            channel.sample_producer.reset_counter();
        }
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
            channel.sample_producer.envelop.envelop_duration_counter = channel.sample_producer.envelop.number_of_envelope_sweep;
            
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

        channel.current_volume = channel.volume;
        channel.timer.update_cycles_to_tick(channel.sample_producer.get_updated_frequency_ticks(channel.frequency));
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