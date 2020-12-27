use super::sample_producer::SampleProducer;
use super::timer::Timer;

pub struct Channel<Procuder: SampleProducer>{
    pub enabled:bool,
    pub frequency:u16,
    pub sound_length:u8,
    pub volume:u8,
    pub trigger:bool,
    pub length_enable:bool,
    pub sample_producer:Procuder,
    pub timer:Timer
}

impl<Procuder: SampleProducer> Channel<Procuder>{
    pub fn new()->Self{
        Channel{
            enabled:false,
            frequency:0,
            sound_length:0,
            volume:0,
            trigger:false,
            length_enable:false,
            sample_producer:Procuder::default(),
            timer: Timer::new(0)
        }   
    }

    pub fn get_audio_sample(&mut self)->f32{
        let sample = if self.timer.cycle(){
            self.sample_producer.produce()
        }
        else{
            0
        };
        
        return if self.enabled{
            Self::convert_digtial_to_analog(sample & self.volume)
        }
        else{
            0.0
        };
    }

    //the formula is y = (2/15)x - 1
    fn convert_digtial_to_analog(sample:u8)->f32{
        let fixed_sample = (sample & 0xF) as f32;

        (2.0/15.0) * fixed_sample - 1.0
    }
}