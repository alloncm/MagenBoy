extern crate lib_gbc;
extern crate wchar;
extern crate winapi;
use lib_gbc::cpu::gbc_cpu::GbcCpu;
use lib_gbc::machine::gbc_memory::GbcMmu;
use lib_gbc::machine::gameboy::GameBoy;
use lib_gbc::opcodes::opcode_resolver::OpcodeResolver;
use lib_gbc::ppu::gbc_ppu::GbcPpu;
use std::ptr;
use wchar::wch_c;
use winapi::ctypes::wchar_t;
use winapi::shared::minwindef::HINSTANCE;

extern "C" {
    fn InitLib(instance: HINSTANCE, name: *const wchar_t);
    fn DrawCycle(colors: *const u32, height: u32, width: u32) -> i32;
}

fn main() {
    
    unsafe{
        
        let mut cpu: GbcCpu = GbcCpu::default();
        let mut mmu: GbcMmu = GbcMmu::default();
        let ppu: GbcPpu = GbcPpu::new(&mmu, &mmu.vram);
        let opcode_resolver = OpcodeResolver::new(&mmu, &cpu.program_counter);
        let gameboy = GameBoy::new(&mut cpu, &mut mmu, opcode_resolver, ppu);
    }

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
