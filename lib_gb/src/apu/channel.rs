use crate::mmu::memory::Memory;
use super::sample_producer::SampleProducer;

pub struct Channel<Procuder: SampleProducer>{
    pub enable:bool,
    pub frequency:u32,
    pub sound_length:Option<u8>,
    pub volume_sweep:Option<u8>,
    pub frequency_sweep:Option<u8>,

    smaple_producer:Procuder
}


impl<Procuder: SampleProducer> Channel<Procuder>{
    pub fn new()->Self{
        Channel{
            enable:false,
            frequency:0,
            sound_length:None,
            frequency_sweep:None,
            volume_sweep:None,
            smaple_producer:Procuder::default()
        }
    
    }

    pub fn get_audio_sample(&self, memory:&dyn Memory)->f32{
        let wave_pattern:[u8;16] = Self::get_wave_patterns(memory);

        

    }

    fn get_wave_patterns(memory:&dyn Memory)->[u8;16]{
        let mut output:[u8;16] = [0;16];

        for i in 0..16{
            output[i] = memory.read(0xFF30 + i as u16);
        }
  
        output
    }

    //the formula is y = (2/15)x - 1
    fn convert_digtial_to_analog(sample:u8)->f32{
        let fixed_sample = (sample & 0xF) as f32;

        (2.0/15.0) * fixed_sample - 1.0
    }
}