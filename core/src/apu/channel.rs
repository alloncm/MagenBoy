use super::{audio_device::{Sample, DEFAULT_SAPMPLE, SAMPLE_MAX}, sample_producer::*, timer::Timer};

pub struct Channel<Procuder: SampleProducer>{
    pub enabled:bool,
    pub frequency:u16,
    pub sound_length:u16,
    pub length_enable:bool,
    pub sample_producer:Procuder,
    pub timer:Timer,

    last_sample:Sample,
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
            last_sample: DEFAULT_SAPMPLE
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

        self.last_sample = DEFAULT_SAPMPLE;
    }

    pub fn get_audio_sample(&mut self)->Sample{
        if self.enabled{
            if self.timer.cycle(){
                self.timer.update_cycles_to_tick(self.sample_producer.get_updated_frequency_ticks(self.frequency));
                let digital_sample = self.sample_producer.produce();
                self.last_sample = Self::convert_digital_to_analog(digital_sample);
            }
        }
        else{
            self.last_sample = DEFAULT_SAPMPLE;
        }
        
        return self.last_sample;
    }

    fn convert_digital_to_analog(digital_sample:u8)->Sample{
        const RATIO:Sample = SAMPLE_MAX / MAX_DIGITAL_SAMPLE as Sample;
        
        // Expandibg the sample to the full range of the Sample type while still 
        // allowing furhter calculations without overflowing
        return (digital_sample as Sample - (MAX_DIGITAL_SAMPLE as Sample / 2)) * RATIO;
    }
}
