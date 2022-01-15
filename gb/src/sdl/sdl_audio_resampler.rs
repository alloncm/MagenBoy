use std::mem::MaybeUninit;
use lib_gb::apu::audio_device::{BUFFER_SIZE, StereoSample};
use sdl2::sys::*;
use crate::audio::audio_resampler::AudioResampler;

pub struct SdlAudioResampler{
    cvt: SDL_AudioCVT
}

impl AudioResampler for SdlAudioResampler{
    fn new(original_frequency:u32, target_frequency:u32)->Self{
        let mut cvt = unsafe{
            let mut cvt:MaybeUninit<SDL_AudioCVT> = MaybeUninit::uninit();
            SDL_BuildAudioCVT(cvt.as_mut_ptr(), AUDIO_S16 as u16, 2, original_frequency as i32,
             AUDIO_S16 as u16, 2, target_frequency as i32);
            cvt.assume_init()
        };

        if cvt.needed != 1{
            std::panic!("Cannot resample between freqs");
        }
        
        cvt.len = (BUFFER_SIZE * std::mem::size_of::<StereoSample>()) as i32;

        log::error!("help");

        Self{cvt}
    }
    
    fn resample(&mut self, buffer:&[StereoSample; BUFFER_SIZE])->Vec<StereoSample>{
        let mut buf:Vec::<u8> = vec![0;(self.cvt.len * self.cvt.len_mult) as usize];
        
        unsafe{
            std::ptr::copy_nonoverlapping(buffer.as_ptr(), buf.as_mut_ptr() as *mut StereoSample, BUFFER_SIZE);
        
            self.cvt.buf = buf.as_mut_ptr();
            let status_code = SDL_ConvertAudio(&mut self.cvt) != 0;
            if status_code{
                std::panic!("error while converting audio, status code: {}", status_code);
            }
        
            let buf_ptr = self.cvt.buf as *mut StereoSample;
            let length = self.cvt.len_cvt as usize / std::mem::size_of::<StereoSample>();
            let mut output = vec![StereoSample::const_defualt();length];

            std::ptr::copy_nonoverlapping(buf_ptr, output.as_mut_ptr(), length);
        
            return output;
        }
    }
}