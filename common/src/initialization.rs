use log::info;

use magenboy_core::{mmu::carts::Mbc, AudioDevice, Bootrom, GameBoy, Mode, GBC_BOOT_ROM_SIZE, GB_BOOT_ROM_SIZE};
#[cfg(feature = "dbg")]
use magenboy_core::debugger::DebuggerInterface;

pub fn check_for_terminal_feature_flag(args:&Vec::<String>, flag:&str)->bool{
    args.len() >= 3 && args.contains(&String::from(flag))
}

pub fn get_terminal_feature_flag_value(args:&Vec<String>, flag:&str, error_message:&str)->String{
    let index = args.iter().position(|v| *v == String::from(flag)).unwrap();
    return args.get(index + 1).expect(error_message).clone();
}

fn get_bootrom_and_mode(args: &Vec<String>, mbc: &mut dyn Mbc) -> (Option<Bootrom>, Mode) {
    let bootrom_path = if check_for_terminal_feature_flag(&args, "--bootrom"){
        Some(get_terminal_feature_flag_value(&args, "--bootrom", "Error! you must specify a value for the --bootrom parameter"))
    }else{
        None
    };

    let bootrom = read_bootrom(bootrom_path);

    let mode = match &bootrom {
        Some(Bootrom::Gb(_)) => Mode::DMG,
        Some(Bootrom::Gbc(_)) => Mode::CGB,
        None => {
            if check_for_terminal_feature_flag(&args, "--mode"){
                let mode = get_terminal_feature_flag_value(&args, "--mode", "Error: Must specify a mode");
                mode.as_str().try_into().expect(format!("Error! mode cannot be: {}", mode).as_str())
            }
            else{
                let mode = mbc.detect_preferred_mode();
                log::info!("Could not find a mode flag, auto detected {}", <Mode as Into<&str>>::into(mode));
                mode
            }
        }
    };

    return (bootrom, mode);
}

#[cfg(not(feature = "dbg"))]
pub fn init_gameboy<'a, A: AudioDevice>(
    args: Vec<String>,
    mbc: &'a mut dyn Mbc,
    audio_devices: A,
) -> GameBoy<'a, A> {
    let (bootrom, mode) = get_bootrom_and_mode(&args, mbc);

    let gameboy = match bootrom{
        Some(b) => GameBoy::new_with_bootrom(mbc, audio_devices, b),
        None => GameBoy::new_with_mode(mbc, audio_devices, mode)
    };

    info!("initialized gameboy successfully!");
    return gameboy;
}

#[cfg(feature = "dbg")]
pub fn init_gameboy<'a, A: AudioDevice, D: DebuggerInterface>(
    args: Vec<String>,
    mbc: &'a mut dyn Mbc,
    audio_devices: A,
    dui: D,
) -> GameBoy<'a, A, D> {
    let (bootrom, mode) = get_bootrom_and_mode(&args, mbc);

    let gameboy = match bootrom{
        Some(b) => GameBoy::new_with_bootrom(mbc, audio_devices, b, dui),
        None => GameBoy::new_with_mode(mbc, audio_devices, mode, dui)
    };

    info!("initialized gameboy successfully!");
    return gameboy;
}

pub fn read_bootrom(bootrom_path: Option<String>) -> Option<Bootrom> {
    bootrom_path.map_or(None, |path| {
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
    })
}