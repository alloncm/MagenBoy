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
    pub timer:Timer,

    last_sample:i8
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
            timer: Timer::new(0),

            last_sample: 0
        }   
    }

    pub fn update_length_register(&mut self){
        if self.length_enable && self.enabled{
            self.sound_length -= 1;
            if self.sound_length == 0{
                self.enabled = false;
                log::warn!("Disabling the channel");
            }
            log::warn!("sl: {}", self.sound_length)
        }
    }

    pub fn get_audio_sample(&mut self)->f32{
        let sample = if self.timer.cycle(){
            self.sample_producer.produce()
        }
        else{
            self.last_sample
        };
        
        self.last_sample = if self.enabled{
             sample
        }
        else{
            0
        };

        self.convert_digtial_to_analog(self.last_sample)
    }

    fn convert_digtial_to_analog(&self, sample:i8)->f32{
        (sample * self.volume as i8) as f32 / 100.0
    }
}