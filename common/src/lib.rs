#![cfg_attr(not(feature = "std"), no_std)]

cfg_if::cfg_if!{ if #[cfg(feature = "std")] {
    pub mod mbc_handler;
    pub mod mpmc_gfx_device;
    pub mod logging;
    pub mod audio{
        mod audio_resampler;
        mod manual_audio_resampler;
        pub use audio_resampler::*;
        pub use manual_audio_resampler::*;
    }
}}

pub mod menu;
pub mod joypad_menu;
pub mod interpolation;

pub const VERSION:&str = env!("CARGO_PKG_VERSION");

cfg_if::cfg_if!{ if #[cfg(feature = "std")] {
    use log::info;

    use magenboy_core::{JoypadProvider, AudioDevice, mmu::{external_memory_bus::Bootrom, GBC_BOOT_ROM_SIZE, GB_BOOT_ROM_SIZE}, GameBoy};
    #[cfg(feature = "dbg")]
    use magenboy_core::debugger::DebuggerInterface;

    use menu::MagenBoyState;        
    use mbc_handler::{initialize_mbc, release_mbc};
    
    pub fn check_for_terminal_feature_flag(args:&Vec::<String>, flag:&str)->bool{
        args.len() >= 3 && args.contains(&String::from(flag))
    }
    
    pub fn get_terminal_feature_flag_value(args:&Vec<String>, flag:&str, error_message:&str)->String{
        let index = args.iter().position(|v| *v == String::from(flag)).unwrap();
        return args.get(index + 1).expect(error_message).clone();
    }
    
    // This is static and not local for the unix signal handler to access it
    pub static EMULATOR_STATE:MagenBoyState = MagenBoyState::new();
    
    pub fn init_and_run(
        args: Vec<String>,
        program_name: String, 
        spsc_gfx_device: mpmc_gfx_device::MpmcGfxDevice, 
        joypad_provider: impl JoypadProvider,
        audio_devices: impl AudioDevice,
        #[cfg(feature = "dbg")] dui: impl DebuggerInterface
    ){
        let bootrom_path = if check_for_terminal_feature_flag(&args, "--bootrom"){
            get_terminal_feature_flag_value(&args, "--bootrom", "Error! you must specify a value for the --bootrom parameter")
        }else{
            String::from("dmg_boot.bin")
        };
        
        let bootrom = match std::fs::read(&bootrom_path){
            Result::Ok(file)=>{
                info!("found bootrom!");
                if file.len() == GBC_BOOT_ROM_SIZE{
                    Some(Bootrom::Gbc(file.try_into().unwrap()))
                }
                else if file.len() == GB_BOOT_ROM_SIZE{
                    Some(Bootrom::Gb(file.try_into().unwrap()))
                }
                else{
                    std::panic!("Error! bootrom: \"{}\" length is invalid", bootrom_path);
                }
            }
            Result::Err(_)=>{
                info!("Could not find bootrom... booting directly to rom");
                Option::None
            }
        };
        
        let mbc = initialize_mbc(&program_name);
        
        let mut gameboy = match bootrom{
            Option::Some(b) => GameBoy::new_with_bootrom(
                mbc, joypad_provider, audio_devices, spsc_gfx_device, b,
                #[cfg(feature = "dbg")] dui),
            Option::None => {
                let mode = if check_for_terminal_feature_flag(&args, "--mode"){
                    let mode = get_terminal_feature_flag_value(&args, "--mode", "Error: Must specify a mode");
                    let mode = mode.as_str().try_into().expect(format!("Error! mode cannot be: {}", mode.as_str()).as_str());
                    mode
                }
                else{
                    std::panic!("Could not infer --mode flag")
                };
                GameBoy::new_with_mode(
                    mbc, joypad_provider, audio_devices, spsc_gfx_device, mode,
                    #[cfg(feature = "dbg")] dui)
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
}}
