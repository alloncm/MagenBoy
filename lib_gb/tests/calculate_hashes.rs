use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use lib_gb::machine::gameboy::GameBoy;
use lib_gb::machine::mbc_initializer::initialize_mbc;
use lib_gb::ppu::gfx_device::GfxDevice;
use lib_gb::apu::audio_device::AudioDevice;
use lib_gb::keypad::joypad_provider::JoypadProvider;

struct StubGfxDevice;
impl GfxDevice for StubGfxDevice{
    fn swap_buffer(&self, buffer:&[u32]) {
        let mut s = DefaultHasher::new();
        buffer.hash(&mut s);
        println!("{}",s.finish());
        std::thread::sleep(std::time::Duration::from_secs(2));
        // std::process::exit(0);
    }
}

struct StubAudioDevice;
impl AudioDevice for StubAudioDevice{
    fn push_buffer(&mut self, _buffer:&[lib_gb::apu::audio_device::Sample]) {}
}

struct StubJoypadProvider;
impl JoypadProvider for StubJoypadProvider{
    fn provide(&mut self, _joypad:&mut lib_gb::keypad::joypad::Joypad) {}
}


#[test]
fn calc_hash(){
    let program = std::fs::read("C:/Users/Alon/Downloads/GameBoy/window_y_trigger.gb")
        .expect("Error could not find file");
    let mut mbc = initialize_mbc(program, None);

    let mut gameboy = GameBoy::new(&mut mbc, StubJoypadProvider{}, StubAudioDevice{}, StubGfxDevice{});

    loop {gameboy.cycle_frame();}

}