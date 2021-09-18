use super::sample_producer::SampleProducer;
use super::timer::Timer;

pub struct Channel<Procuder: SampleProducer>{
    pub enabled:bool,
    pub frequency:u16,
    pub sound_length:u16,
    pub length_enable:bool,
    pub sample_producer:Procuder,
    pub timer:Timer,

    last_sample:f32,
}

impl<Procuder: SampleProducer> Channel<Procuder>{
    pub fn new(sample_producer:Procuder)->Self{
        Channel{
            enabled:false,
            frequency:0,
            sound_length:0,
            length_enable:false,
            timer: Timer::new(sample_producer.get_updated_frequency_ticks(0)),
            sample_producer,
            last_sample: 0.0
        }   
    }

    pub fn update_length_register(&mut self){
        if self.length_enable{
            if self.sound_length > 0{
                self.sound_length -= 1;
            }
            if self.sound_length == 0{
                self.enabled = false;
            }
        }
    }

    pub fn reset(&mut self){
        self.enabled = false;
        self.frequency = 0;
        self.length_enable = false;
        self.sound_length = 0;
        self.timer.update_cycles_to_tick(self.sample_producer.get_updated_frequency_ticks(self.frequency));
        self.sample_producer.reset();

        self.last_sample = 0.0;
    }

    pub fn get_audio_sample(&mut self)->f32{
        if self.enabled{

            let sample = if self.timer.cycle(){
                self.timer.update_cycles_to_tick(self.sample_producer.get_updated_frequency_ticks(self.frequency));
                let s = self.sample_producer.produce();
                self.convert_digtial_to_analog(s)
            }
            else{
                self.last_sample
            };

            self.last_sample = sample;
    
            return self.last_sample;
        }
        
        return 0.0;
    }

    fn convert_digtial_to_analog(&self, sample:u8)->f32{
        (sample as f32 / 7.5 ) - 1.0
    }
}
