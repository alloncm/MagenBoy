use super::sample_producer::SampleProducer;
use super::freq_sweep::FreqSweep;
use super::volume_envelop::VolumeEnvlope;

pub struct ToneSweepSampleProducer{
    pub wave_duty:u8,
    pub sweep:FreqSweep,
    pub envelop:VolumeEnvlope,

    duty_sample_pointer:u8,
}

const DUTY_TABLE:[[u8; 8]; 4] = [
    [0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,1],
    [1,0,0,0,0,1,1,1],
    [0,1,1,1,1,1,1,0],
];

impl Default for ToneSweepSampleProducer{
    fn default()->Self{
        ToneSweepSampleProducer{
            wave_duty:1,
            sweep:FreqSweep{
                sweep_shift:0,
                sweep_decrease:false,
                time_sweep:0,
                shadow_frequency:0
            },
            envelop:VolumeEnvlope{
                increase_envelope:false,
                number_of_envelope_sweep:0,
                envelop_duration_counter:0
            },
            duty_sample_pointer:0
        }
    }
}

impl SampleProducer for ToneSweepSampleProducer{

    fn produce(&mut self)->u8{
        if self.duty_sample_pointer >= 8{
            self.duty_sample_pointer = 0;
        }

        let sample = DUTY_TABLE[self.wave_duty as usize][self.duty_sample_pointer as usize];

        self.duty_sample_pointer += 1;

        return sample;
    }
}

