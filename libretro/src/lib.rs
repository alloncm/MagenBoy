mod devices;
mod logging;

use std::{ffi::{c_char, c_uint, c_void}, mem::MaybeUninit, ptr::null_mut, slice};

use libretro_sys::*;

use magenboy_core::{machine::{gameboy::GameBoy, mbc_initializer}, mmu::external_memory_bus::Bootrom, ppu::gb_ppu::*};

use crate::{devices::*, logging::*};

pub struct MagenBoyRetroCore<'a>{
    gameboy: Option<GameBoy<'a, RetroJoypadProvider,  RetroAudioDevice, RetroGfxDevice>>,
    save_data_fat_ptr: Option<(*mut u8, usize)>,
    video_cb: Option<VideoRefreshFn>,
    audio_cb: Option<AudioSampleBatchFn>,
    input_poll_cb: Option<InputPollFn>,
    input_cb: Option<InputStateFn>,
    environment_cb: Option<EnvironmentFn>
}
pub(crate) static mut RETRO_CORE_CTX: MagenBoyRetroCore = MagenBoyRetroCore{
    gameboy: None, save_data_fat_ptr: None, video_cb: None, audio_cb: None, input_poll_cb: None, input_cb: None, environment_cb: None,
};

#[no_mangle]
pub unsafe extern "C" fn retro_get_system_info(system_info: *mut SystemInfo){
    const NAME:*const c_char = b"MagenBoy\0".as_ptr() as _;
    const VERSION_SIZE:usize = magenboy_common::VERSION.len() + 1;
    const VERSION:[u8;VERSION_SIZE] = {
        let mut result:[u8;VERSION_SIZE] = [0;VERSION_SIZE];
        let mut i = 0;
        while i < VERSION_SIZE - 1{
            result[i] = magenboy_common::VERSION.as_bytes()[i];
            i +=1 ;
        }
        result[VERSION_SIZE - 1] = b'\0';
        result
    };
    const VALID_EXTENSIONS:*const c_char = b"gb|gbc\0".as_ptr() as _;

    std::ptr::write_bytes(system_info, 0, 1);   //memset
    (*system_info).library_name = NAME;
    (*system_info).library_version = VERSION.as_ptr() as *const c_char;
    (*system_info).need_fullpath = false;
    (*system_info).valid_extensions = VALID_EXTENSIONS;
    (*system_info).block_extract = false;
}

#[no_mangle]
pub unsafe extern "C" fn retro_get_system_av_info(av_info: *mut SystemAvInfo){
    std::ptr::write_bytes(av_info, 0, 1);   //memset
    (*av_info).timing.fps = 60.0;
    (*av_info).timing.sample_rate = RetroAudioDevice::OUTPUT_FREQUENCY as f64;
    (*av_info).geometry.base_width = SCREEN_WIDTH as c_uint;
    (*av_info).geometry.base_height = SCREEN_HEIGHT as c_uint;
    (*av_info).geometry.max_width = SCREEN_WIDTH as c_uint;
    (*av_info).geometry.max_height = SCREEN_HEIGHT as c_uint;
    (*av_info).geometry.aspect_ratio = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
}

#[no_mangle] pub extern "C" fn retro_api_version()->c_uint{API_VERSION}
#[no_mangle] pub extern "C" fn retro_get_region()->c_uint{Region::NTSC.to_uint()}
#[no_mangle] pub unsafe extern "C" fn retro_set_video_refresh(cb: VideoRefreshFn){RETRO_CORE_CTX.video_cb = Some(cb)}
#[no_mangle] pub unsafe extern "C" fn retro_set_audio_sample_batch(cb: AudioSampleBatchFn){RETRO_CORE_CTX.audio_cb = Some(cb)}
#[no_mangle] pub unsafe extern "C" fn retro_set_input_poll(cb: InputPollFn){RETRO_CORE_CTX.input_poll_cb = Some(cb)}
#[no_mangle] pub unsafe extern "C" fn retro_set_input_state(cb: InputStateFn){RETRO_CORE_CTX.input_cb = Some(cb)}
#[no_mangle] pub unsafe extern "C" fn retro_set_environment(cb:EnvironmentFn){RETRO_CORE_CTX.environment_cb = Some(cb)}

#[no_mangle] 
pub unsafe extern "C" fn retro_init(){
    let mut log_cb = MaybeUninit::<LogCallback>::uninit();
    if (RETRO_CORE_CTX.environment_cb.unwrap())(ENVIRONMENT_GET_LOG_INTERFACE, log_cb.as_mut_ptr() as *mut c_void){
        let log_cb = log_cb.assume_init();
        RetroLogger::init(log::LevelFilter::Trace, Some(log_cb));
        log::info!("Init logger successfully");
    }
    else{
        RetroLogger::init(log::LevelFilter::Off, None);
    }
}

#[no_mangle]
pub unsafe extern "C" fn retro_load_game(game_info: *const GameInfo)->bool{
    let rom_buffer = slice::from_raw_parts::<u8>((*game_info).data as *const u8, (*game_info).size);
    let mbc = mbc_initializer::initialize_mbc(rom_buffer, None, None);
    if mbc.has_battery(){
        RETRO_CORE_CTX.save_data_fat_ptr = Some((mbc.get_ram().as_mut_ptr(), mbc.get_ram().len()));
    }
    RETRO_CORE_CTX.gameboy = Some(GameBoy::new(mbc, RetroJoypadProvider, RetroAudioDevice::default(), RetroGfxDevice, Bootrom::None, None));
    
    let mut pixel_format = PixelFormat::RGB565.to_uint();
    if !(RETRO_CORE_CTX.environment_cb.unwrap())(ENVIRONMENT_SET_PIXEL_FORMAT, &mut pixel_format as *mut u32 as *mut c_void){
        log::error!("RGB565 is not supported, can't initialize MagenBoy");
        return false;
    }

    log::info!("Load game and initiated magenboy successfully");
    return true;
}

#[no_mangle] 
pub unsafe extern "C" fn retro_get_memory_data(id:c_uint)->*mut c_void{
    return match id{
        MEMORY_SAVE_RAM => match RETRO_CORE_CTX.save_data_fat_ptr {
            Some(ptr) => ptr.0 as *mut c_void,
            _ => null_mut(),
        },
        _=> null_mut(),
    };
}

#[no_mangle] 
pub unsafe extern "C" fn retro_get_memory_size(id:c_uint)->isize{
    return match id{
        MEMORY_SAVE_RAM => match RETRO_CORE_CTX.save_data_fat_ptr {
            Some(ptr) => ptr.1 as isize,
            _ => 0,
        },
        _=> 0,
    };
}

#[no_mangle]
pub unsafe extern "C" fn retro_run(){
    RETRO_CORE_CTX.gameboy.as_mut().unwrap().cycle_frame();
    RetroAudioDevice::push_audio_buffer_to_libretro();
}

#[no_mangle] pub extern "C" fn retro_load_game_special(_:c_uint, _:*const GameInfo, _:isize)->bool{false}
#[no_mangle] pub extern "C" fn retro_serialize_size()->isize{0}
#[no_mangle] pub extern "C" fn retro_serialize(_:*mut c_void, _:isize)->bool{false}
#[no_mangle] pub extern "C" fn retro_unserialize(_:*const c_void, _:isize)->bool{false}
#[no_mangle] pub extern "C" fn retro_deinit(){}
#[no_mangle] pub extern "C" fn retro_unload_game(){}
#[no_mangle] pub extern "C" fn retro_reset(){}
#[no_mangle] pub extern "C" fn retro_set_audio_sample(_: AudioSampleFn){}
#[no_mangle] pub extern "C" fn retro_set_controller_port_device(_: c_uint, _:c_uint){}
#[no_mangle] pub extern "C" fn retro_cheat_reset(){}
#[no_mangle] pub extern "C" fn retro_cheat_set(_: c_uint, _:bool, _:*const c_char){}