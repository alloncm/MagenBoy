use super::channel::Channel;
use super::wave_sample_producer::WaveSampleProducer;
use crate::mmu::memory::Memory;


pub struct GbApu{
    pub wave_channel:Channel<WaveSampleProducer>,

    
    current_cycle:u32
}

impl Default for GbApu{
    fn default() -> Self {
        GbApu{
            wave_channel:Channel::<WaveSampleProducer>::new(),
            current_cycle:0
        }
    }
}

impl GbApu{
    pub fn cycle(&mut self, memory: &dyn Memory, cycles_passed:u8)->Vec<f32>{
        let samples:Vec<f32> = vec![0.0 ; cycles_passed as usize];

        for i in 0..cycles_passed{
            samples[i as usize] = self.wave_channel.get_audio_sample(memory);
        }

        samples
    }
}

