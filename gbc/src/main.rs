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
use lib_gbc::machine::rom::Rom;

extern "C" {
    fn InitLib(instance: HINSTANCE, name: *const wchar_t);
    fn DrawCycle(colors: *const u32, height: u32, width: u32) -> i32;
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = match fs::read(&args[1]){
        Result::Ok(val)=>val,
        Result::Err(why)=>panic!("could not read file {}",why)
    };
    let rom = Rom::new(file);

    let mut gameboy = GameBoy::new(rom);

    unsafe {
        //let name: *const u16 = wch_c!("test").as_ptr();
        //InitLib(ptr::null_mut(), name);
        //let colors: [u32; 50 * 50] = [0x50505050; 50 * 50];
        loop {
            gameboy.cycle();
            /*if DrawCycle(colors.as_ptr(), 50, 50) == 0 {
                break;
            }*/
        }
    }
}
