use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Read;
use lib_gb::apu::audio_device::BUFFER_SIZE;
use lib_gb::ppu::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use lib_gb::ppu::gfx_device::Pixel;
use lib_gb::{
    apu::audio_device::AudioDevice, keypad::joypad_provider::JoypadProvider,
    machine::{gameboy::GameBoy, mbc_initializer::initialize_mbc}, ppu::gfx_device::GfxDevice
};

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
    run_integration_test_from_url(file_url, 800, 3798827046966939676);
}

#[test]
fn test_cpu_instrs_timing(){
    let file_url = "https://raw.githubusercontent.com/retrio/gb-test-roms/master/instr_timing/instr_timing.gb";
    run_integration_test_from_url(file_url, 100, 469033992149587554);
}

#[test]
fn test_dmg_acid(){
    let file_url = "https://github.com/mattcurrie/dmg-acid2/releases/download/v1.0/dmg-acid2.gb";
    run_integration_test_from_url(file_url, 60, 1690571533691915665);
}

#[test]
fn test_turtle_window_y_trigger(){
    run_turtle_integration_test("window_y_trigger.gb", 15511617103807079362);
}

#[test]
fn test_turtle_window_y_trigger_wx_offscreen(){
    run_turtle_integration_test("window_y_trigger_wx_offscreen.gb", 15592061677463553443);
}

fn run_turtle_integration_test(program_name:&str, hash:u64){
    let zip_url = "https://github.com/Powerlated/TurtleTests/releases/download/v1.0/release.zip";
    
    let file = reqwest::blocking::get(zip_url).unwrap()
        .bytes().unwrap();

    let cursor = std::io::Cursor::new(file.as_ref());

    let mut programs = zip::ZipArchive::new(cursor).unwrap();
    let zip_file = programs.by_name(program_name).unwrap();
    let program = zip_file.bytes().map(|x|x.unwrap()).collect::<Vec<u8>>();

    run_integration_test(program, 100, hash, format!("The program: {} has failed", program_name));
}

fn run_integration_test_from_url(program_url:&str, frames_to_execute:u32, expected_hash:u64){
    let file = reqwest::blocking::get(program_url).unwrap()
        .bytes().unwrap();

    let program = Vec::from(file.as_ref());
    let fail_message = format!("The program {} has failed", program_url);
    run_integration_test(program, frames_to_execute, expected_hash, fail_message);
}

fn run_integration_test(program:Vec<u8>, frames_to_execute:u32, expected_hash:u64, fail_message:String){
    let mut mbc = initialize_mbc(program, None);
    let mut last_hash:u64 = 0;
    let mut found = false;
    let mut gameboy = GameBoy::new(
        &mut mbc,
        StubJoypadProvider{},
        StubAudioDevice{}, 
        CheckHashGfxDevice{hash:expected_hash,last_hash_p:&mut last_hash, found_p:&mut found}
    );

    for _ in 0..frames_to_execute {
        gameboy.cycle_frame();
        if found{
            return;
        }
    }
    assert!(false, "{}", fail_message);   
}



// This function is for clcualting the hash of a new test rom
/// # Examples
///
///```
///#[test]
///fn calc_custom_rom_hash(){
///    calc_hash("path_to_rom");
///}
///```

#[test]
#[ignore]
fn generate_hash(){
    let path = "path to rom";
    calc_hash(path);
}

fn calc_hash(rom_path:&str){
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
                    std::process::exit(0);
                }
                LAST_HASH = hash;
            }
        }
    }

    let program = std::fs::read(rom_path)
        .expect("Error could not find file");
    
    let program = Vec::from(program);

    let mut mbc = initialize_mbc(program, None);

    let mut gameboy = GameBoy::new(&mut mbc, StubJoypadProvider{}, StubAudioDevice{}, GetHashGfxDevice{});

    loop {gameboy.cycle_frame();}
}