extern crate wchar;
extern crate winapi;
extern crate lib_gbc;
use std::ptr;
use wchar::wch_c;
use winapi::ctypes::wchar_t;
use winapi::shared::minwindef::HINSTANCE;
use lib_gbc::cpu::GbcCpu::GbcCpu;

extern "C" {
    fn InitLib(instance: HINSTANCE, name: *const wchar_t);
    fn DrawCycle(colors: *const u32, height: u32, width: u32) -> i32;
}

fn main() {

    let cpu:GbcCpu = GbcCpu::default();
    cpu.af();


    unsafe {
        let name: *const u16 = wch_c!("test").as_ptr();
        InitLib(ptr::null_mut(), name);
        let colors: [u32; 50 * 50] = [0x50505050; 50 * 50];
        loop {
            if DrawCycle(colors.as_ptr(), 50, 50) == 0 {
                break;
            }
        }
    }
}
