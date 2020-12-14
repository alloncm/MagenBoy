use lib_gb::apu::audio_device::AudioDevice;
use stupid_gfx::audio::Audio;
use stupid_gfx::initializer::Initializer;

const GB_SOUND_FREQUENCY:u32 = 4_194_304;

pub struct StupidGfxAudioDevie{
    device:Audio,
    to_skip:u32
}

impl StupidGfxAudioDevie{
    pub fn new(stupid_initializer:&Initializer,frequency:u32, fps:u16, channels:u8)->Self{
        let to_skip = GB_SOUND_FREQUENCY / frequency;
        if to_skip == 0{
            std::panic!("freqency is too high: {}", frequency);
        }

        StupidGfxAudioDevie{
            device:stupid_initializer.init_audio(frequency as i32, channels, fps),
            to_skip: to_skip
        }
    }
}

impl AudioDevice for StupidGfxAudioDevie{
    fn push_buffer(&self, buffer:&[f32]){
        let mut filtered_buffer:Vec<f32> = Vec::new();
        let mut counter = 0;
        for sample in buffer.into_iter(){
            if *sample != 0.0{
                //println!("{}", sample);
            }
            
            if counter == self.to_skip{
                filtered_buffer.push(*sample);
                counter = 0;
            }
            else{
                counter += 1;
            }
        }

        self.device.push_audio_to_device(&filtered_buffer).unwrap();
    }
}