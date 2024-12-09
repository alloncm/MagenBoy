use log::info;

use magenboy_core::{AudioDevice, Bootrom, GameBoy, JoypadProvider, Mode, GBC_BOOT_ROM_SIZE, GB_BOOT_ROM_SIZE};
#[cfg(feature = "dbg")]
use magenboy_core::debugger::DebuggerInterface;

use crate::{mbc_handler::{initialize_mbc, release_mbc}, menu::MagenBoyState, mpmc_gfx_device::MpmcGfxDevice};        

pub fn check_for_terminal_feature_flag(args:&Vec::<String>, flag:&str)->bool{
    args.len() >= 3 && args.contains(&String::from(flag))
}

pub fn get_terminal_feature_flag_value(args:&Vec<String>, flag:&str, error_message:&str)->String{
    let index = args.iter().position(|v| *v == String::from(flag)).unwrap();
    return args.get(index + 1).expect(error_message).clone();
}

// This is static and not local for the unix signal handler to access it
pub static EMULATOR_STATE:MagenBoyState = MagenBoyState::new();

pub fn init_and_run_gameboy(
    args: Vec<String>,
    program_name: String, 
    spsc_gfx_device: MpmcGfxDevice, 
    joypad_provider: impl JoypadProvider,
    audio_devices: impl AudioDevice,
    #[cfg(feature = "dbg")] dui: impl DebuggerInterface
){
    let bootrom_path = if check_for_terminal_feature_flag(&args, "--bootrom"){
        Some(get_terminal_feature_flag_value(&args, "--bootrom", "Error! you must specify a value for the --bootrom parameter"))
    }else{
        None
    };

    let bootrom = bootrom_path.map_or(None, |path| {
        match std::fs::read(&path){
            Result::Ok(file)=>{
                info!("found bootrom!");
                match file.len() {
                    GBC_BOOT_ROM_SIZE => Some(Bootrom::Gbc(file.try_into().unwrap())),
                    GB_BOOT_ROM_SIZE => Some(Bootrom::Gb(file.try_into().unwrap())),
                    _=> std::panic!("Error! bootrom: \"{}\" length is invalid", path)
                }
            }
            Result::Err(_)=>{
                info!("Could not find bootrom... booting directly to rom");
                None
            }
        }
    });

    let mbc = initialize_mbc(&program_name);

    let mut gameboy = match bootrom{
        Some(b) => GameBoy::new_with_bootrom(mbc, joypad_provider, audio_devices, spsc_gfx_device, b, #[cfg(feature = "dbg")] dui),
        None => {
            let mode = if check_for_terminal_feature_flag(&args, "--mode"){
                let mode = get_terminal_feature_flag_value(&args, "--mode", "Error: Must specify a mode");
                let mode = mode.as_str().try_into().expect(format!("Error! mode cannot be: {}", mode).as_str());
                mode
            }
            else{
                let mode = mbc.detect_prefered_mode();
                log::info!("Could not find a mode flag, auto detected {}", <Mode as Into<&str>>::into(mode));
                mode
            };
            GameBoy::new_with_mode(mbc, joypad_provider, audio_devices, spsc_gfx_device, mode, #[cfg(feature = "dbg")] dui)
        }
    };

    info!("initialized gameboy successfully!");

    EMULATOR_STATE.running.store(true, std::sync::atomic::Ordering::Relaxed);
    while EMULATOR_STATE.running.load(std::sync::atomic::Ordering::Relaxed){
        if !EMULATOR_STATE.pause.load(std::sync::atomic::Ordering::SeqCst){
            let state = &EMULATOR_STATE;
            let _mutex_ctx = state.state_mutex.lock().unwrap();
            gameboy.cycle_frame();
        }
    }
    drop(gameboy);
    release_mbc(&program_name, mbc);
    log::info!("released the gameboy succefully");
}