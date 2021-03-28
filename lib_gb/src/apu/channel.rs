use super::sample_producer::SampleProducer;
use super::timer::Timer;

pub struct Channel<Procuder: SampleProducer>{
    pub enabled:bool,
    pub frequency:u16,
    pub sound_length:u8,
    pub volume:u8,
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
            length_enable:false,
            sample_producer:Procuder::default(),
            timer: Timer::new(0),

            last_sample: 0
        }   
    }

    pub fn update_length_register(&mut self){
        if self.enabled{
            if self.sound_length > 0{
                self.sound_length -= 1;
            }
            if self.sound_length == 0 && self.length_enable{
                self.enabled = false;
            }
        }
    }

    pub fn reset(&mut self){
        self.enabled = false;
        self.frequency = 0;
        self.length_enable = false;
        self.sound_length = 0;
        self.timer.update_cycles_to_tick(0);
        self.volume = 0;

        self.last_sample = 0;
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