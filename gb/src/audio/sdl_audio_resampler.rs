use std::mem::MaybeUninit;
use lib_gb::apu::audio_device::{BUFFER_SIZE, StereoSample};
use sdl2::sys::*;
use super::AudioResampler;

pub struct SdlAudioResampler{
    original_frequency:u32,
    target_frequency:u32,
}

impl AudioResampler for SdlAudioResampler{
    fn new(original_frequency:u32, target_frequency:u32)->Self{
        Self{
            original_frequency,
            target_frequency,
        }
    }
    
    fn resample(&mut self, buffer:&[StereoSample; BUFFER_SIZE])->Vec<StereoSample>{
        unsafe{
            let mut cvt = {
                let mut cvt:MaybeUninit<SDL_AudioCVT> = MaybeUninit::uninit();
                SDL_BuildAudioCVT(cvt.as_mut_ptr(), AUDIO_S16 as u16, 2, self.original_frequency as i32,
                 AUDIO_S16 as u16, 2, self.target_frequency as i32);
                cvt.assume_init()
            };
    
            if cvt.needed != 1{
                std::panic!("Cannot resample between freqs");
            }
            
            cvt.len = (BUFFER_SIZE * std::mem::size_of::<StereoSample>()) as i32;
            let mut buf:Vec::<u8> = vec![0;(cvt.len * cvt.len_mult) as usize];

            std::ptr::copy_nonoverlapping(buffer.as_ptr(), buf.as_mut_ptr() as *mut StereoSample, BUFFER_SIZE);
        
            cvt.buf = buf.as_mut_ptr();
            if SDL_ConvertAudio(&mut cvt) != 0{
                std::panic!("error while converting audio, status code: {}", status_code);
            }
        
            let buf_ptr = cvt.buf as *mut StereoSample;
            let length = cvt.len_cvt as usize / std::mem::size_of::<StereoSample>();
            let mut output = vec![StereoSample::const_defualt();length];

            std::ptr::copy_nonoverlapping(buf_ptr, output.as_mut_ptr(), length);
        
            return output;
        }
    }
}