use std::{ffi::c_void, mem::MaybeUninit};
use lib_gb::{GB_FREQUENCY, apu::audio_device::*};
use super::{AudioResampler, ResampledAudioDevice, get_sdl_error_message};

use sdl2::sys::*;
use crossbeam_channel::{Receiver, SendError, Sender, bounded};


struct UserData{
    rx: Receiver<[Sample;BUFFER_SIZE]>,
    current_buf: Option<[Sample;BUFFER_SIZE]>,
    current_buf_index:usize,
}

pub struct SdlPullAudioDevice<AR:AudioResampler>{
    resampler: AR,
    buffer: [Sample;BUFFER_SIZE],
    buffer_index:usize,

    tarnsmiter: Sender<[Sample;BUFFER_SIZE]>,

    userdata: UserData
}

impl<AR:AudioResampler> SdlPullAudioDevice<AR>{
    fn push_audio_to_device(&self, audio:&[Sample; BUFFER_SIZE])->Result<(), SendError<[Sample; BUFFER_SIZE]>>{
        self.tarnsmiter.send(audio.clone())
    }
}

impl<AR:AudioResampler> ResampledAudioDevice<AR> for SdlPullAudioDevice<AR>{
    fn new(frequency:i32, turbo_mul:u8)->Self{

        // cap of less than 2 hurts the fps
        let(s,r) = bounded(2);
        let data = UserData{
            current_buf:Option::None,
            current_buf_index:0,
            rx:r
        };

        let mut device = SdlPullAudioDevice{
            buffer:[DEFAULT_SAPMPLE;BUFFER_SIZE],
            buffer_index:0,
            resampler: AudioResampler::new(GB_FREQUENCY * turbo_mul as u32, frequency as u32),
            tarnsmiter:s,
            userdata:data
        };
        
        let desired_audio_spec = SDL_AudioSpec{
            freq: frequency,
            format: AUDIO_S16SYS as u16,
            channels: 2,
            silence: 0,
            samples: BUFFER_SIZE as u16,
            padding: 0,
            size: 0,
            callback: Option::Some(audio_callback),
            userdata: (&mut device.userdata) as *mut UserData as *mut c_void
        };

        
        let mut uninit_audio_spec:MaybeUninit<SDL_AudioSpec> = MaybeUninit::uninit();

        unsafe{
            SDL_Init(SDL_INIT_AUDIO);
            SDL_ClearError();
            let id = SDL_OpenAudioDevice(std::ptr::null(), 0, &desired_audio_spec, uninit_audio_spec.as_mut_ptr() , 0);

            if id == 0{
                std::panic!("{}", get_sdl_error_message());
            }

            let init_audio_spec:SDL_AudioSpec = uninit_audio_spec.assume_init();

            if init_audio_spec.freq != frequency {
                std::panic!("Error initializing audio could not use the frequency: {}", frequency);
            }

            //This will start the audio processing
            SDL_PauseAudioDevice(id, 0);
        };

        return device;
    }
    fn get_audio_buffer(&mut self) ->(&mut [Sample;BUFFER_SIZE], &mut usize) {
        (&mut self.buffer, &mut self.buffer_index)
    }
    fn get_resampler(&mut self) ->&mut AR {
        &mut self.resampler
    }
    fn full_buffer_callback(&self) ->Result<(), String> {
        self.push_audio_to_device(&self.buffer).map_err(|e|e.to_string())
    }
}

impl<AR:AudioResampler> AudioDevice for SdlPullAudioDevice<AR>{
    fn push_buffer(&mut self, buffer:&[StereoSample; BUFFER_SIZE]) {
        ResampledAudioDevice::push_buffer(self, buffer);
    }
}

unsafe extern "C" fn audio_callback(userdata:*mut c_void, buffer:*mut u8, length:i32){
    let length = length as usize;
    let safe_userdata = &mut *(userdata as *mut UserData);

    if safe_userdata.current_buf.is_none(){
        safe_userdata.current_buf = Some(safe_userdata.rx.recv().unwrap());
    }

    let samples = safe_userdata.current_buf.unwrap();
    let samples_size = (samples.len() * std::mem::size_of::<Sample>()) - safe_userdata.current_buf_index;
    let samples_ptr = (samples.as_ptr() as *mut u8).add(safe_userdata.current_buf_index);
    std::ptr::copy_nonoverlapping(samples_ptr, buffer, std::cmp::min(length, samples_size));

    if length > samples_size && safe_userdata.rx.is_empty(){
        safe_userdata.current_buf = Option::None;
        safe_userdata.current_buf_index = 0;
        std::ptr::write_bytes(buffer.add(samples.len() as usize), 0, length  - samples_size);
    }
    else if length > samples_size{
        safe_userdata.current_buf = Option::None;
        safe_userdata.current_buf_index = 0;
        audio_callback(userdata, buffer.add(samples_size), (length - samples_size) as i32);
    }
    else{
        safe_userdata.current_buf_index = length;
    }
}

