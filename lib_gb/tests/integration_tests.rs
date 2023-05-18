use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};
use std::io::Read;
use lib_gb::machine::Mode;
use lib_gb::mmu::{external_memory_bus::Bootrom, carts::Mbc};
use lib_gb::ppu::{gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::{Pixel, GfxDevice}};
use lib_gb::apu::audio_device::{BUFFER_SIZE, AudioDevice};
use lib_gb::keypad::joypad_provider::JoypadProvider;
use lib_gb::machine::{gameboy::GameBoy, mbc_initializer::initialize_mbc};

struct CheckHashGfxDevice{
    hash:u64,
    last_hash_p:*mut u64,
    found_p:*mut bool,
}
impl GfxDevice for CheckHashGfxDevice{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        let mut s = DefaultHasher::new();
        buffer.hash(&mut s);
        let hash = s.finish();
        unsafe{
            if *self.last_hash_p == hash && hash == self.hash{
                println!("{}", hash);
                *self.found_p = true;
            }
            *self.last_hash_p = hash;
        }
    }
}

struct StubAudioDevice;
impl AudioDevice for StubAudioDevice{
    fn push_buffer(&mut self, _buffer:&[lib_gb::apu::audio_device::StereoSample; BUFFER_SIZE]) {}
}

struct StubJoypadProvider;
impl JoypadProvider for StubJoypadProvider{
    fn provide(&mut self, _joypad:&mut lib_gb::keypad::joypad::Joypad) {}
}

#[test]
fn test_cpu_instrs(){
    let file_url = "https://raw.githubusercontent.com/retrio/gb-test-roms/master/cpu_instrs/cpu_instrs.gb";
    run_integration_test_from_url(file_url, 800, 3798827046966939676, Some(Mode::DMG));
}

#[test]
fn test_cpu_instrs_timing(){
    let file_url = "https://raw.githubusercontent.com/retrio/gb-test-roms/master/instr_timing/instr_timing.gb";
    run_integration_test_from_url(file_url, 100, 469033992149587554, Some(Mode::DMG));
}

#[test]
fn test_dmg_acid(){
    let file_url = "https://github.com/mattcurrie/dmg-acid2/releases/download/v1.0/dmg-acid2.gb";
    run_integration_test_from_url(file_url, 60, 1690571533691915665, Some(Mode::DMG));
}

#[test]
fn test_turtle_window_y_trigger(){
    run_turtle_integration_test("window_y_trigger.gb", 15511617103807079362);
}

#[test]
fn test_turtle_window_y_trigger_wx_offscreen(){
    run_turtle_integration_test("window_y_trigger_wx_offscreen.gb", 15592061677463553443);
}

#[test]
fn test_mooneye_acceptance_ppu_intr_2_0_timing(){
    run_mooneye_test_suite_test("acceptance/ppu/intr_2_0_timing.gb", 10509154465546589481);
}

#[test]
fn test_mooneye_acceptance_ppu_intr_2_mode0_timing(){
    run_mooneye_test_suite_test("acceptance/ppu/intr_2_mode0_timing.gb", 13181382744438017604);
}

#[test]
fn test_mooneye_acceptance_ppu_intr_2_mode3_timing(){
    run_mooneye_test_suite_test("acceptance/ppu/intr_2_mode3_timing.gb", 6495990171031472337);
}

#[test]
fn test_mooneye_acceptance_ppu_intr_2_oam_ok_timing(){
    run_mooneye_test_suite_test("acceptance/ppu/intr_2_oam_ok_timing.gb", 1784377789505089325);
}

#[test]
fn test_magentests_bg_oam_priority(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.1.2/bg_oam_priority.gbc";
    run_integration_test_from_url(file_url, 60, 14781023578080553257, Some(Mode::CGB));
}

#[test]
fn test_magentests_oam_internal_priority(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.2.0/oam_internal_priority.gbc";
    run_integration_test_from_url(file_url, 60, 9280650417949081747, Some(Mode::CGB));
}

#[test]
fn test_cgb_acid2(){
    let file_url = "https://github.com/mattcurrie/cgb-acid2/releases/download/v1.1/cgb-acid2.gbc";
    run_integration_test_from_url(file_url, 60, 16716513650835367856, Some(Mode::CGB));
}

fn run_turtle_integration_test(program_name:&str, hash:u64){
    let zip_url = "https://github.com/Powerlated/TurtleTests/releases/download/v1.0/release.zip";
    let program = get_ziped_program(zip_url, program_name);
    run_integration_test(program, Bootrom::None, 100, hash, format!("The program: {} has failed", program_name), Some(Mode::DMG));
}

fn run_mooneye_test_suite_test(program_name:&str, hash:u64){
    let zip_url = "https://gekkio.fi/files/mooneye-test-suite/mts-20220522-1522-55c535c/mts-20220522-1522-55c535c.zip";
    let boot_rom_url = "https://github.com/alloncm/MagenBoot/releases/download/0.1.1/dmg_boot.bin";
    let program_zip_path = format!("{}/{program_name}", "mts-20220522-1522-55c535c");
    let program = get_ziped_program(zip_url, program_zip_path.as_str());
    let boot_rom = reqwest::blocking::get(boot_rom_url).unwrap().bytes().unwrap().to_vec();
    run_integration_test(program, Bootrom::Gb(boot_rom.try_into().unwrap()), 300, hash, format!("The program: {} has failed", program_zip_path), Some(Mode::DMG));
}

fn get_ziped_program(zip_url:&str, program_zip_path:&str)->Vec<u8>{
    let zip_file = reqwest::blocking::get(zip_url).unwrap().bytes().unwrap();
    let cursor = std::io::Cursor::new(zip_file.as_ref());
    let mut programs = zip::ZipArchive::new(cursor).unwrap();
    let zip_file = programs.by_name(program_zip_path).unwrap();
    let program = zip_file.bytes().map(|x|x.unwrap()).collect::<Vec<u8>>();
    return program;
}

fn run_integration_test_from_url(program_url:&str, frames_to_execute:u32, expected_hash:u64, mode:Option<Mode>){
    let file = reqwest::blocking::get(program_url).unwrap().bytes().unwrap();
    let program = Vec::from(file.as_ref());
    let fail_message = format!("The program {} has failed", program_url);
    run_integration_test(program, Bootrom::None, frames_to_execute, expected_hash, fail_message, mode);
}

fn run_integration_test(program:Vec<u8>, boot_rom:Bootrom, frames_to_execute:u32, expected_hash:u64, fail_message:String, mode:Option<Mode>){
    let mbc:&'static mut dyn Mbc = initialize_mbc(&program, None, mode);
    let mut last_hash:u64 = 0;
    let mut found = false;
    let mut gameboy = GameBoy::new(
        mbc,
        StubJoypadProvider{},
        StubAudioDevice{}, 
        CheckHashGfxDevice{hash:expected_hash,last_hash_p:&mut last_hash, found_p:&mut found},
        boot_rom,
        mode
    );

    for _ in 0..frames_to_execute {
        gameboy.cycle_frame();
        if found{
            return;
        }
    }
    assert!(false, "{}", fail_message);
}



// This function is for calculating the hash of a new test rom
/// # Examples
///
///```
///#[test]
///fn calc_custom_rom_hash(){
///    calc_hash("path_to_rom", None /*in case no bootrom needed*/);
///}
///```

#[test]
#[ignore]
fn generate_hash(){
    let path = "path to rom";
    let boot_rom_path = None;
    let mode = Some(Mode::DMG);
    calc_hash(path, boot_rom_path, mode);
}

fn calc_hash(rom_path:&str, boot_rom_path:Option<&str>, mode:Option<Mode>){
    static mut FRAMES_COUNTER:u32 = 0;
    static mut LAST_HASH:u64 = 0;
    struct GetHashGfxDevice;
    impl GfxDevice for GetHashGfxDevice{
        fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
            unsafe{
                if FRAMES_COUNTER < 700{
                    FRAMES_COUNTER += 1;
                    return;
                }
            }
            let mut s = DefaultHasher::new();
            buffer.hash(&mut s);
            let hash = s.finish();
            unsafe{
                if LAST_HASH == hash{
                    println!("{}", hash);
                    let mut vec_buffer = Vec::<u8>::new();
                    for i in 0..buffer.len(){
                        vec_buffer.push((buffer[i] & 0xFF) as u8);
                        vec_buffer.push((buffer[i] >> 8 & 0xFF) as u8);
                        vec_buffer.push((buffer[i] >> 16 & 0xFF) as u8);
                    }
                    image::save_buffer("output.bmp", &vec_buffer, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, image::ColorType::Rgb8).unwrap();
                    std::process::exit(0);
                }
                LAST_HASH = hash;
            }
        }
    }

    let program = std::fs::read(rom_path)
        .expect("Error could not find file");
    
    let program = Vec::from(program);

    let mbc = initialize_mbc(&program, None, mode);

    let mut gameboy = if let Some(boot_rom_path) = boot_rom_path{
        let boot_rom = std::fs::read(boot_rom_path).expect("Cant find bootrom");
        GameBoy::new(mbc, StubJoypadProvider{}, StubAudioDevice{}, GetHashGfxDevice{}, Bootrom::Gb(boot_rom.try_into().unwrap()), mode)
    }
    else{
        GameBoy::new(mbc, StubJoypadProvider{}, StubAudioDevice{}, GetHashGfxDevice{}, Bootrom::None, mode)
    };

    loop {gameboy.cycle_frame();}
}