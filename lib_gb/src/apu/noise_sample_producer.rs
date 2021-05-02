use crate::utils::bit_masks::set_bit_u16;

use super::{sample_producer::SampleProducer, volume_envelop::VolumeEnvlope};

pub struct NoiseSampleProducer {
    pub envelop: VolumeEnvlope,
    pub lfsr: u16,
    pub bits_to_shift_divisor: u8,
    pub width_mode: bool,
    pub divisor_code: u8,
}

impl Default for NoiseSampleProducer {
    fn default() -> Self {
        Self {
            envelop: VolumeEnvlope::default(),
            divisor_code: 0,
            width_mode: false,
            bits_to_shift_divisor: 0,
            lfsr: 0,
        }
    }
}

impl SampleProducer for NoiseSampleProducer {
    //Step the scranble opertaion one step.
    fn produce(&mut self) -> u8 {
        let xor_result = (self.lfsr & 0b01) ^ ((self.lfsr & 0b10) >> 1);
        self.lfsr >>= 1;
        self.lfsr |= xor_result << 14;

        if self.width_mode {
            set_bit_u16(&mut self.lfsr, 6, false);
            self.lfsr |= xor_result << 6;
        }

        let sample = ((!self.lfsr) & 1) as u8;

        return sample * self.envelop.current_volume;
    }

    fn reset(&mut self) {
        self.lfsr = 0;
        self.width_mode = false;
        self.bits_to_shift_divisor = 0;
        self.divisor_code = 0;
        self.envelop.reset();
    }

    fn get_updated_frequency_ticks(&self, _freq: u16) -> u16 {
        //Divider code 0 is treated as 8
        let divisor: u16 = if self.divisor_code == 0 {
            8
        } else {
            // equals to deivisor_code * 16
            (self.divisor_code as u16) << 4
        };

        divisor << self.bits_to_shift_divisor
    }
}
