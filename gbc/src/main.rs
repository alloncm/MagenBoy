extern crate lib_gbc;
extern crate wchar;
extern crate winapi;
use lib_gbc::machine::gameboy::GameBoy;
use std::ptr;
use wchar::wch_c;
use winapi::ctypes::wchar_t;
use winapi::shared::minwindef::HINSTANCE;
use std::fs;
use std::env;
use std::result::Result;
use std::vec::Vec;
use lib_gbc::mmu::mbc_initializer::initialize_mbc;

const BOOT_ROM_SIZE:usize = 0xFF;

extern "C" {
    fn InitLib(instance: HINSTANCE, name: *const wchar_t);
    fn DrawCycle(colors: *const u32, height: u32, width: u32) -> i32;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file = match fs::read("Dependencies\\Init\\dmg_boot.bin"){
        Result::Ok(val)=>val,
        Result::Err(why)=>panic!("could not read file {}",why)
    };
    
    let mut bootrom:[u8;BOOT_ROM_SIZE] = [0;BOOT_ROM_SIZE];
    for i in 0..BOOT_ROM_SIZE{
        bootrom[i] = file[i];
    }

    let program = match fs::read(&args[1]){
        Result::Ok(val)=>val,
        Result::Err(why)=>panic!("could not read file {}",why)
    };
    

    let mbc = initialize_mbc(program);    

    let mut gameboy = GameBoy::new(mbc, bootrom);

    unsafe {
        let name: *const u16 = wch_c!("test").as_ptr();
        InitLib(ptr::null_mut(), name);
        loop {
            
            for _ in 0..17000{
                gameboy.cycle();
            }
            let vec = gameboy.get_screen_buffer();
            if DrawCycle(vec.as_ptr() as *const u32, 144, 160) == 0 {
                break;
            }
        }
    }
}
