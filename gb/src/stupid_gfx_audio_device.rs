use std::vec::Vec;
use lib_gb::apu::audio_device::AudioDevice;
use stupid_gfx::audio::Audio;
use stupid_gfx::initializer::Initializer;

const GB_SOUND_FREQUENCY:u32 = 4_194_304;
const BUFFER_SIZE:usize = 1024;

pub struct StupidGfxAudioDevie{
    device:Audio,
    to_skip:u32,

    buffer: Vec<f32>
}

impl StupidGfxAudioDevie{
    pub fn new(stupid_initializer:&Initializer,frequency:u32, channels:u8)->Self{
        let to_skip = GB_SOUND_FREQUENCY / frequency;
        if to_skip == 0{
            std::panic!("freqency is too high: {}", frequency);
        }

        StupidGfxAudioDevie{
            device:stupid_initializer.init_audio(frequency as i32, channels, 1024),
            to_skip: to_skip,
            buffer: Vec::with_capacity(BUFFER_SIZE)
        }
    }
}

impl AudioDevice for StupidGfxAudioDevie{
    fn push_buffer(&mut self, buffer:&[f32]){
        let mut counter = 0;
        for sample in buffer.into_iter(){
            if *sample != 0.0{
                //log::info!("{}", sample)
            }
            
            if counter == self.to_skip{
                self.buffer.push(*sample);
                counter = 0;

                if self.buffer.len() == BUFFER_SIZE{
                    self.device.push_audio_to_device(&self.buffer).unwrap();
                    self.buffer.clear();
                }
            }
            else{
                counter += 1;
            }
        }
    }
}