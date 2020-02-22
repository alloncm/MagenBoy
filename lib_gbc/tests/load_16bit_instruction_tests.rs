extern crate lib_gbc;
use lib_gbc::cpu::gbc_cpu::{GbcCpu};
use lib_gbc::opcodes::load_16bit_instructions;

#[test]
fn test_ld_hl_sp_dd(){
    let opcode:u16 = 0x23;
    let mut cpu = GbcCpu::default();
    load_16bit_instructions::ld_hl_spdd(&mut cpu, opcode);
    assert_eq!(cpu.hl.value, opcode);
}