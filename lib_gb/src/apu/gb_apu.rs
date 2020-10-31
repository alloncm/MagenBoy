use super::channel::Channel;
use super::wave_sample_producer::WaveSampleProducer;
use super::audio_device::AudioDevice;

pub const AUDIO_BUFFER_SIZE:usize = 0x100;

pub struct GbApu<Device: AudioDevice>{
    pub wave_channel:Channel<WaveSampleProducer>,

    audio_buffer:[f32;AUDIO_BUFFER_SIZE],
    current_cycle:u32,
    device:Device
}

impl<Device: AudioDevice> GbApu<Device>{
    pub fn new(device: Device) -> Self {
        GbApu{
            wave_channel:Channel::<WaveSampleProducer>::new(),
            audio_buffer:[0.0; AUDIO_BUFFER_SIZE],
            current_cycle:0,
            device:device
        }
    }

    pub fn cycle(&mut self, cycles_passed:u8){
        

        for i in 0..cycles_passed{
            self.audio_buffer[i as usize] = self.wave_channel.get_audio_sample();
        }
    }
}

