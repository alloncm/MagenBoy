use std::{collections::hash_map::DefaultHasher, convert::TryInto, hash::{Hash, Hasher}, io::Read, sync::atomic::AtomicBool};

use magenboy_core::{keypad::{joypad::Joypad, joypad_provider::JoypadProvider}, machine::{Mode, gameboy::GameBoy, mbc_initializer::initialize_mbc}, mmu::{external_memory_bus::Bootrom, carts::Mbc}, ppu::{gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, gfx_device::*}, apu::audio_device::*};

struct CheckHashGfxDevice<'a>{
    hash: u64,
    last_hash: u64,
    found: &'a AtomicBool,
}
impl<'a> GfxDevice for CheckHashGfxDevice<'a>{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        let mut s = DefaultHasher::new();
        buffer.hash(&mut s);
        let hash = s.finish();
        if self.last_hash == hash && hash == self.hash{
            self.found.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.last_hash = hash;
    }
}

struct StubAudioDevice;
impl AudioDevice for StubAudioDevice{
    fn push_buffer(&mut self, _buffer:&[StereoSample; BUFFER_SIZE]) {}
}

struct StubJoypadProvider;
impl JoypadProvider for StubJoypadProvider{
    fn provide(&mut self, _joypad:&mut Joypad) {}
}

#[test]
fn test_cpu_instrs(){
    let file_url = "https://raw.githubusercontent.com/retrio/gb-test-roms/master/cpu_instrs/cpu_instrs.gb";
    run_integration_test_from_url(file_url, 3200, 15560803699908721371, Some(Mode::DMG));
}

#[test]
fn test_cpu_instrs_timing(){
    let file_url = "https://raw.githubusercontent.com/retrio/gb-test-roms/master/instr_timing/instr_timing.gb";
    run_integration_test_from_url(file_url, 100, 6688493151528556732, Some(Mode::DMG));
}

#[test]
fn test_dmg_acid_dmg_mode(){
    let file_url = "https://github.com/mattcurrie/dmg-acid2/releases/download/v1.0/dmg-acid2.gb";
    run_integration_test_from_url(file_url, 60, 1467713036241655344, Some(Mode::DMG));
}

#[test]
fn test_dmg_acid_cgb_mode(){
    let file_url = "https://github.com/mattcurrie/dmg-acid2/releases/download/v1.0/dmg-acid2.gb";
    run_integration_test_from_url(file_url, 60, 18025850858500536480, Some(Mode::CGB));
}

#[test]
fn test_turtle_window_y_trigger(){
    run_turtle_integration_test("window_y_trigger.gb", 6465875958237578550);
}

#[test]
fn test_turtle_window_y_trigger_wx_offscreen(){
    run_turtle_integration_test("window_y_trigger_wx_offscreen.gb", 18187429968502985545);
}

#[test]
fn test_mooneye_acceptance_ppu_intr_2_0_timing(){
    run_mooneye_test_suite_test("acceptance/ppu/intr_2_0_timing.gb", 7475320393161591745);
}

#[test]
fn test_mooneye_acceptance_ppu_intr_2_mode0_timing(){
    run_mooneye_test_suite_test("acceptance/ppu/intr_2_mode0_timing.gb", 9052326526940620337);
}

#[test]
fn test_mooneye_acceptance_ppu_intr_2_mode3_timing(){
    run_mooneye_test_suite_test("acceptance/ppu/intr_2_mode3_timing.gb", 14127472135696903085);
}

#[test]
fn test_mooneye_acceptance_ppu_intr_2_oam_ok_timing(){
    run_mooneye_test_suite_test("acceptance/ppu/intr_2_oam_ok_timing.gb", 14374012711624871933);
}

#[test]
fn test_magentests_bg_oam_priority(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.3.0/bg_oam_priority.gbc";
    run_integration_test_from_url(file_url, 60, 10888561623649800478, Some(Mode::CGB));
}

#[test]
fn test_magentests_oam_internal_priority(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.2.0/oam_internal_priority.gbc";
    run_integration_test_from_url(file_url, 60, 3314422793898507891, Some(Mode::CGB));
}

#[test]
fn test_magentests_hblank_vram_dma(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.3.0/hblank_vram_dma.gbc";
    run_integration_test_from_url(file_url, 60, 6410113756445583331, Some(Mode::CGB));
}

#[test]
fn test_magentests_key0_lock_after_boot(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.4.0/key0_lock_after_boot.gbc";
    run_integration_test_from_url(file_url, 60, 6410113756445583331, Some(Mode::CGB));
}

#[test]
fn test_cgb_acid2(){
    let file_url = "https://github.com/mattcurrie/cgb-acid2/releases/download/v1.1/cgb-acid2.gbc";
    run_integration_test_from_url(file_url, 60, 1123147979104076695, Some(Mode::CGB));
}

#[test]
fn test_magentests_ppu_off_stat_reg_state_cgb_mode(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.5.0/ppu_disabled_state.gbc";
    run_integration_test_from_url(file_url, 60, 6410113756445583331, Some(Mode::CGB));
}

#[test]
fn test_magentests_ppu_off_stat_reg_state_dmg_mode(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.5.0/ppu_disabled_state.gbc";
    run_integration_test_from_url(file_url, 60, 3114957375595162924, Some(Mode::DMG));
}

#[test]
fn test_magentests_mbc1_oob_access_cgb_mode(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.5.0/mbc_oob_sram_mbc1.gbc";
    run_integration_test_from_url(file_url, 60, 6410113756445583331, Some(Mode::CGB));
}

#[test]
fn test_magentests_mbc1_oob_access_dmg_mode(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.5.0/mbc_oob_sram_mbc1.gbc";
    run_integration_test_from_url(file_url, 60, 3114957375595162924, Some(Mode::DMG));
}

#[test]
fn test_magentests_mbc3_oob_access_cgb_mode(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.5.0/mbc_oob_sram_mbc3.gbc";
    run_integration_test_from_url(file_url, 60, 6410113756445583331, Some(Mode::CGB));
}

#[test]
fn test_magentests_mbc3_oob_access_dmg_mode(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.5.0/mbc_oob_sram_mbc3.gbc";
    run_integration_test_from_url(file_url, 60, 3114957375595162924, Some(Mode::DMG));
}

#[test]
fn test_magentests_mbc5_oob_access_cgb_mode(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.5.0/mbc_oob_sram_mbc5.gbc";
    run_integration_test_from_url(file_url, 60, 6410113756445583331, Some(Mode::CGB));
}

#[test]
fn test_magentests_mbc5_oob_access_dmg_mode(){
    let file_url = "https://github.com/alloncm/MagenTests/releases/download/0.5.0/mbc_oob_sram_mbc5.gbc";
    run_integration_test_from_url(file_url, 60, 3114957375595162924, Some(Mode::DMG));
}

fn run_turtle_integration_test(program_name:&str, hash:u64){
    let zip_url = "https://github.com/Powerlated/TurtleTests/releases/download/v1.0/release.zip";
    let program = get_ziped_program(zip_url, program_name);
    run_integration_test(program, None, 100, hash, format!("The program: {} has failed", program_name), Some(Mode::DMG));
}

fn run_mooneye_test_suite_test(program_name:&str, hash:u64){
    let zip_url = "https://gekkio.fi/files/mooneye-test-suite/mts-20220522-1522-55c535c/mts-20220522-1522-55c535c.zip";
    let boot_rom_url = "https://github.com/alloncm/MagenBoot/releases/download/0.1.1/dmg_boot.bin";
    let program_zip_path = format!("{}/{program_name}", "mts-20220522-1522-55c535c");
    let program = get_ziped_program(zip_url, program_zip_path.as_str());
    let boot_rom = reqwest::blocking::get(boot_rom_url).unwrap().bytes().unwrap().to_vec();
    run_integration_test(program, Some(Bootrom::Gb(boot_rom.try_into().unwrap())), 300, hash, format!("The program: {} has failed", program_zip_path), Some(Mode::DMG));
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
    run_integration_test(program, None, frames_to_execute, expected_hash, fail_message, mode);
}

fn run_integration_test(program:Vec<u8>, boot_rom:Option<Bootrom>, frames_to_execute:u32, expected_hash:u64, fail_message:String, mode:Option<Mode>){
    let mbc:&'static mut dyn Mbc = initialize_mbc(&program, None);
    let found = AtomicBool::new(false);
    let mut gameboy = match boot_rom {
        Some(b)=>GameBoy::new_with_bootrom(
            mbc,
            StubJoypadProvider{},
            StubAudioDevice{}, 
            CheckHashGfxDevice{hash:expected_hash,last_hash: 0, found: &found}, b),
        None => GameBoy::new_with_mode(mbc,
            StubJoypadProvider{},
            StubAudioDevice{}, 
            CheckHashGfxDevice{hash:expected_hash,last_hash: 0, found: &found}, mode.unwrap())
        };

    for _ in 0..frames_to_execute {
        gameboy.cycle_frame();
        if found.load(std::sync::atomic::Ordering::Relaxed){
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
    let path = r"C:\Users\Alon\source\MagenTests\build\ppu_disabled_state.gbc";
    let boot_rom_path = None;
    let mode = Some(Mode::DMG);
    calc_hash(path, boot_rom_path, mode);
}

fn calc_hash(rom_path:&str, boot_rom_path:Option<&str>, mode:Option<Mode>){
    struct GetHashGfxDevice{
        last_hash:u64,
        last_hash_counter:u32,
        frames_counter:u32
    }
    impl GfxDevice for GetHashGfxDevice{
        fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
            if self.frames_counter < 700{
                self.frames_counter += 1;
                return;
            }
            let mut s = DefaultHasher::new();
            buffer.hash(&mut s);
            let hash = s.finish();
            if self.last_hash == hash{
                self.last_hash_counter += 1;
                if self.last_hash_counter > 600{
                    std::fs::write("calc_hash_output.txt", hash.to_string().as_bytes()).unwrap();
                    let buffer = buffer
                        .map(|c|[(((c >> 11) & 0b1_1111) as u8) << 3, (((c >> 5) & 0b11_1111) as u8) << 2, (((c) & 0b1_1111) as u8) << 3])
                        .concat();
                    image::save_buffer("calc_hash_output.bmp", &buffer, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, image::ColorType::Rgb8).unwrap();
                    std::process::exit(0);
                }
            }
            else{
                self.last_hash_counter = 0;
                self.last_hash = hash;
            }
        }
    }

    let program = std::fs::read(rom_path).expect("Error could not find file");
    
    let program = Vec::from(program);

    let mbc = initialize_mbc(&program, None);

    let test_gfx_device = GetHashGfxDevice{ last_hash: 0, last_hash_counter: 0, frames_counter: 0 };
    let mut gameboy = if let Some(boot_rom_path) = boot_rom_path{
        let boot_rom = std::fs::read(boot_rom_path).expect("Cant find bootrom");
        GameBoy::new_with_bootrom(mbc, StubJoypadProvider{}, StubAudioDevice{}, test_gfx_device,Bootrom::Gb(boot_rom.try_into().unwrap()))
    }
    else{
        GameBoy::new_with_mode(mbc, StubJoypadProvider{}, StubAudioDevice{}, test_gfx_device, mode.unwrap())
    };

    loop {gameboy.cycle_frame();}
}