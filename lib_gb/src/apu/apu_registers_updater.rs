use crate::{mmu::io_ports::*, utils::bit_masks::*};

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


pub fn set_nr41(channel:&mut Channel<NoiseSampleProducer>, value:u8){
    let length_data = value & 0b11_1111;
    channel.sound_length = 64 - length_data as u16
}

pub fn set_nr42(channel:&mut Channel<NoiseSampleProducer>, value:u8){
    update_volume_envelope( value, &mut channel.sample_producer.envelop);
    if !is_dac_enabled(channel.sample_producer.envelop.volume, channel.sample_producer.envelop.increase_envelope){
        channel.enabled = false;
    }
}

pub fn set_nr43(channel:&mut Channel<NoiseSampleProducer>, nr43:u8){
    channel.sample_producer.bits_to_shift_divisor = (nr43 & 0b1111_0000) >> 4;
    channel.sample_producer.width_mode = (nr43 & BIT_3_MASK) != 0;
    channel.sample_producer.divisor_code = nr43 & 0b111;
}

pub fn set_nr44(channel:&mut Channel<NoiseSampleProducer>, fs:&FrameSequencer, nr44:u8){
    let dac_enabled = is_dac_enabled(channel.sample_producer.envelop.volume, channel.sample_producer.envelop.increase_envelope);
    update_channel_conrol_register(channel, dac_enabled, nr44, 64, fs);
    if (nr44 & BIT_7_MASK) != 0{
        //On trigger all the LFSR bits are set (lfsr is 15 bit register)
        channel.sample_producer.lfsr = 0x7FFF;

        channel.sample_producer.envelop.current_volume = channel.sample_producer.envelop.volume;
    }
}

pub fn set_nr50<AD:AudioDevice>(apu:&mut GbApu<AD>, nr50:u8){
    apu.right_terminal.enabled = nr50 & BIT_3_MASK != 0;
    apu.left_terminal.enabled = nr50 & BIT_7_MASK != 0;
    
    apu.right_terminal.volume = nr50 & 0b111;
    apu.left_terminal.volume = (nr50 & 0b111_0000) >> 4;
}


pub fn set_nr51<AD:AudioDevice>(apu:&mut GbApu<AD>, nr51:u8){
    for i in 0..NUMBER_OF_CHANNELS{
        apu.right_terminal.channels[i as usize] = nr51 & (1 << i) != 0;
    }
    for i in 0..NUMBER_OF_CHANNELS{
        apu.left_terminal.channels[i as usize] = nr51 & (0b1_0000 << i) != 0;
    }
}

pub fn set_nr52<AD:AudioDevice>(apu:&mut GbApu<AD>, ports:&mut [u8;IO_PORTS_SIZE], nr52:u8){
    apu.enabled = nr52 & BIT_7_MASK != 0;

    for i in NR10_REGISTER_INDEX..NR52_REGISTER_INDEX{
        ports[i as usize] = 0;
    }
}

pub fn get_nr52<AD:AudioDevice>(apu:&GbApu<AD>, nr52:&mut u8){
    set_bit_u8(nr52, 3, apu.noise_channel.enabled && apu.noise_channel.length_enable && apu.noise_channel.sound_length != 0);
    set_bit_u8(nr52, 2, apu.wave_channel.enabled && apu.wave_channel.length_enable && apu.wave_channel.sound_length != 0);
    set_bit_u8(nr52, 1, apu.tone_channel.enabled && apu.tone_channel.length_enable && apu.tone_channel.sound_length != 0);
    set_bit_u8(nr52, 0, apu.sweep_tone_channel.enabled && apu.sweep_tone_channel.length_enable && apu.sweep_tone_channel.sound_length != 0);
}

pub fn set_nr30(channel:&mut Channel<WaveSampleProducer>, value:u8){
    if (value & BIT_7_MASK) == 0{
        channel.enabled = false;
    }
}

pub fn set_nr31(channel:&mut Channel<WaveSampleProducer>, value:u8){
    channel.sound_length = 256 - (value as u16);
}

pub fn set_nr32(channel:&mut Channel<WaveSampleProducer>, nr32:u8){
    //I want bits 5-6
    channel.sample_producer.volume = (nr32 & 0b110_0000) >> 5;
}

pub fn set_nr33(channel:&mut Channel<WaveSampleProducer>, nr33:u8){
    //discard lower 8 bits
    channel.frequency &= 0xFF00;
    channel.frequency |= nr33 as u16;
}

pub fn set_nr34(channel:&mut Channel<WaveSampleProducer>, fs:&FrameSequencer, nr30:u8, nr34:u8){
    //clear the upper 8 bits
    channel.frequency &= 0xFF;
    channel.frequency |= ((nr34 & 0b111) as u16) << 8;

    let dac_enabled = (nr30 & BIT_7_MASK) != 0;
    update_channel_conrol_register(channel, dac_enabled, nr34, 256, fs);

    if nr34 & BIT_7_MASK != 0{
        channel.sample_producer.reset_counter();
    }
}

pub fn set_nr10(channel:&mut Channel<SquareSampleProducer>, value:u8){
    let sweep = channel.sample_producer.sweep.as_mut().unwrap();
    sweep.sweep_decrease = (value & 0b1000) != 0;
    sweep.sweep_shift = value & 0b111;
    sweep.sweep_period = (value & 0b111_0000) >> 4;
}

pub fn set_nr11(channel:&mut Channel<SquareSampleProducer>, value:u8){    
    channel.sample_producer.wave_duty = (value & 0b1100_0000) >> 6;
    channel.sound_length = 64 - (value & 0b11_1111) as u16
}
 pub fn set_nr12(channel:&mut Channel<SquareSampleProducer>, value:u8){
    update_volume_envelope(value, &mut channel.sample_producer.envelop);
        
    if !is_dac_enabled(channel.sample_producer.envelop.volume, channel.sample_producer.envelop.increase_envelope){
        channel.enabled = false;
    }
}

 pub fn set_nr13(channel:&mut Channel<SquareSampleProducer>, value:u8){
    //discard lower bits
    channel.frequency &= 0xFF00;
    channel.frequency |= value as u16;
}

pub fn set_nr14(channel:&mut Channel<SquareSampleProducer>, fs:&FrameSequencer, nr14:u8){
    //discard upper bits
    channel.frequency &= 0xFF;
    channel.frequency |= ((nr14 & 0b111) as u16) << 8;
    
    let dac_enabled = is_dac_enabled(channel.sample_producer.envelop.volume, channel.sample_producer.envelop.increase_envelope);
    update_channel_conrol_register(channel, dac_enabled, nr14, 64, fs);

    if nr14 & BIT_7_MASK != 0{
        //volume
        channel.sample_producer.envelop.envelop_duration_counter = channel.sample_producer.envelop.number_of_envelope_sweep;
        channel.sample_producer.envelop.current_volume = channel.sample_producer.envelop.volume;
        
        //sweep
        let sweep = channel.sample_producer.sweep.as_mut().unwrap();
        sweep.channel_trigger(channel.frequency);
        if sweep.sweep_shift > 0{
            let freq = sweep.calculate_new_frequency();
            channel.enabled = !FreqSweep::check_overflow(freq);
        }
    
    }
}

pub fn set_nr24(channel:&mut Channel<SquareSampleProducer>, fs:&FrameSequencer, nr14:u8){
    //discard upper bits
    channel.frequency &= 0xFF;
    channel.frequency |= ((nr14 & 0b111) as u16) << 8;
    
    let dac_enabled = is_dac_enabled(channel.sample_producer.envelop.volume, channel.sample_producer.envelop.increase_envelope);
    update_channel_conrol_register(channel, dac_enabled, nr14, 64, fs);

    if nr14 & BIT_7_MASK != 0{
        //volume
        channel.sample_producer.envelop.envelop_duration_counter = channel.sample_producer.envelop.number_of_envelope_sweep;
        channel.sample_producer.envelop.current_volume = channel.sample_producer.envelop.volume;
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

        channel.timer.update_cycles_to_tick(channel.sample_producer.get_updated_frequency_ticks(channel.frequency));
    }
}

fn update_volume_envelope(register:u8, envelop:&mut VolumeEnvlope){
    envelop.volume = (register & 0b1111_0000) >> 4;
    envelop.number_of_envelope_sweep = register & 0b111;
    envelop.increase_envelope = (register & BIT_3_MASK) != 0;
}

fn is_dac_enabled(volume:u8, envelop_increase:bool)->bool{
    volume != 0 || envelop_increase
}