use crate::{mmu::{gb_mmu::GbMmu, memory::UnprotectedMemory, io_ports::*}, utils::{bit_masks::*, memory_registers::*}};

use super::{
    audio_device::AudioDevice, 
    channel::Channel, 
    frame_sequencer::FrameSequencer, 
    freq_sweep::FreqSweep, 
    gb_apu::GbApu, 
    noise_sample_producer::NoiseSampleProducer, 
    sample_producer::SampleProducer, 
    square_sample_producer::SquareSampleProducer, 
    volume_envelop::VolumeEnvlope, 
    wave_sample_producer::WaveSampleProducer,
    sound_utils::NUMBER_OF_CHANNELS
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

fn prepare_tone_channel(channel:&mut Channel<SquareSampleProducer>, memory:&mut GbMmu,fs:&FrameSequencer){ 

    if memory.io_ports.get_ports_cycle_trigger()[NR21_REGISTER_INDEX as usize]{
        channel.sound_length = 64 - (memory.read_unprotected(NR21_REGISTER_ADDRESS) & 0b11_1111) as u16;
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR22_REGISTER_INDEX as usize]{
        update_volume_envelope(&mut channel.volume, memory.read_unprotected(NR22_REGISTER_ADDRESS), &mut channel.sample_producer.envelop);
        
        if !is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope){
            channel.enabled = false;
        }
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR23_REGISTER_INDEX as usize]{
        //discard lower bit
        channel.frequency &= 0xFF00;
        channel.frequency |= memory.read_unprotected(NR23_REGISTER_ADDRESS) as u16;
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR24_REGISTER_INDEX as usize]{
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

    if memory.io_ports.get_ports_cycle_trigger()[NR41_REGISTER_INDEX as usize]{
        let length_data = memory.read_unprotected(NR41_REGISTER_ADDRESS) & 0b11_1111;
        channel.sound_length = 64 - length_data as u16
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR42_REGISTER_INDEX as usize]{
        update_volume_envelope(&mut channel.volume, memory.read_unprotected(NR42_REGISTER_ADDRESS), &mut channel.sample_producer.envelop);
        if !is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope){
            channel.enabled = false;
        }
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR43_REGISTER_INDEX as usize]{
        let nr43 = memory.read_unprotected(NR43_REGISTER_ADDRESS);
        channel.sample_producer.bits_to_shift_divisor = (nr43 & 0b1111_0000) >> 4;
        channel.sample_producer.width_mode = (nr43 & BIT_3_MASK) != 0;
        channel.sample_producer.divisor_code = nr43 & 0b111;
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR44_REGISTER_INDEX as usize]{
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
    let channel_control = memory.read_unprotected(NR50_REGISTER_ADDRESS);
    apu.right_terminal.enabled = channel_control & BIT_3_MASK != 0;
    apu.left_terminal.enabled = channel_control & BIT_7_MASK != 0;
    
    apu.right_terminal.volume = channel_control & 0b111;
    apu.left_terminal.volume = (channel_control & 0b111_0000) >> 4;

    let channels_output_terminals = memory.read_unprotected(NR51_REGISTER_ADDRESS);

    for i in 0..NUMBER_OF_CHANNELS{
        apu.right_terminal.channels[i as usize] = channels_output_terminals & (1 << i) != 0;
    }
    for i in 0..NUMBER_OF_CHANNELS{
        apu.left_terminal.channels[i as usize] = channels_output_terminals & (0b1_0000 << i) != 0;
    }

    let master_sound = memory.read_unprotected(NR52_REGISTER_ADDRESS);
    apu.enabled = master_sound & BIT_7_MASK != 0;
}

fn prepare_wave_channel(channel:&mut Channel<WaveSampleProducer>, memory:&mut GbMmu,fs:&FrameSequencer){

    if memory.io_ports.get_ports_cycle_trigger()[NR30_REGISTER_INDEX as usize]{
        if (memory.read_unprotected(NR30_REGISTER_ADDRESS) & BIT_7_MASK) == 0{
            channel.enabled = false;
        }
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR31_REGISTER_INDEX as usize]{
        channel.sound_length = 256 - (memory.read_unprotected(NR31_REGISTER_ADDRESS) as u16);
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR32_REGISTER_INDEX as usize]{
        //I want bits 5-6
        let nr32 = memory.read_unprotected(NR32_REGISTER_ADDRESS);
        channel.sample_producer.volume = (nr32 & 0b110_0000) >> 5;
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR33_REGISTER_INDEX as usize]{
        //discard lower 8 bits
        channel.frequency &= 0xFF00;
        channel.frequency |= memory.read_unprotected(NR33_REGISTER_ADDRESS) as u16;
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR34_REGISTER_INDEX as usize]{
        let nr34 = memory.read_unprotected(NR34_REGISTER_ADDRESS);

        //clear the upper 8 bits
        channel.frequency &= 0xFF;
        channel.frequency |= ((nr34 & 0b111) as u16) << 8;

        let dac_enabled = (memory.read_unprotected(NR30_REGISTER_ADDRESS) & BIT_7_MASK) != 0;
        update_channel_conrol_register(channel, dac_enabled, nr34, 256, fs);

        //Since in the wave channel the volume is shifted and managed by the sampler producer
        //the channel current_volume - which the DAC uses, should always be one.
        channel.current_volume = 1;

        if nr34 & BIT_7_MASK != 0{
            channel.sample_producer.reset_counter();
        }
    }

    for i in 0..=0xF{
        channel.sample_producer.wave_samples[i] = memory.read_unprotected(0xFF30 + i as u16);
    }
}

fn prepare_tone_sweep_channel(channel:&mut Channel<SquareSampleProducer>, memory:&mut GbMmu, fs:&FrameSequencer){
    let nr10 = memory.read_unprotected(NR10_REGISTER_ADDRESS);
    let nr11 = memory.read_unprotected(NR11_REGISTER_ADDRESS);
    let nr12 = memory.read_unprotected(NR12_REGISTER_ADDRESS);
    let nr13 = memory.read_unprotected(NR13_REGISTER_ADDRESS);
    let nr14 = memory.read_unprotected(NR14_REGISTER_ADDRESS);

    if memory.io_ports.get_ports_cycle_trigger()[NR10_REGISTER_INDEX as usize]{
        //sweep
        let sweep = channel.sample_producer.sweep.as_mut().unwrap();
        sweep.sweep_decrease = (nr10 & 0b1000) != 0;
        sweep.sweep_shift = nr10 & 0b111;
        sweep.sweep_period = (nr10 & 0b111_0000) >> 4;
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR11_REGISTER_INDEX as usize]{
        channel.sample_producer.wave_duty = (nr11 & 0b1100_0000) >> 6;
        channel.sound_length = 64 - (nr11 & 0b11_1111) as u16
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR12_REGISTER_INDEX as usize]{
        update_volume_envelope(&mut channel.volume, nr12, &mut channel.sample_producer.envelop);
        
        if !is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope){
            channel.enabled = false;
        }
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR13_REGISTER_INDEX as usize]{
        //discard lower bits
        channel.frequency &= 0xFF00;
        channel.frequency |= nr13 as u16;
    }
    if memory.io_ports.get_ports_cycle_trigger()[NR14_REGISTER_INDEX as usize]{
        //discard upper bits
        channel.frequency &= 0xFF;
        channel.frequency |= ((nr14 & 0b111) as u16) << 8;
        
        let dac_enabled = is_dac_enabled(channel.volume, channel.sample_producer.envelop.increase_envelope);
        update_channel_conrol_register(channel, dac_enabled, nr14, 64, fs);

        if nr14 & BIT_7_MASK != 0{
            //volume
            channel.sample_producer.envelop.envelop_duration_counter = channel.sample_producer.envelop.number_of_envelope_sweep;
            
            //sweep
            let sweep = channel.sample_producer.sweep.as_mut().unwrap();
            sweep.channel_trigger(channel.frequency);
            if sweep.sweep_shift > 0{
                let freq = sweep.calculate_new_frequency();
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