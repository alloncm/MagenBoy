use magenboy_core::cpu::{flag::Flag, opcodes::arithmetic_16bit_instructions, gb_cpu::GbCpu};

#[test]
fn test_add_sp_dd_positive_dd(){
    let mut cpu = GbCpu::default();
    let opcode:u16 = 88;
    arithmetic_16bit_instructions::add_sp_dd(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 88);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),false);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
}

#[test]
fn test_add_sp_dd(){
    let mut cpu = GbCpu::default();
    cpu.stack_pointer =0xFFF8;
    let opcode:u16 = 2;
    arithmetic_16bit_instructions::add_sp_dd(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 0xFFFA);
    assert_eq!(cpu.get_flag(Flag::Carry),false);
    assert_eq!(cpu.get_flag(Flag::HalfCarry),false);
    assert_eq!(cpu.get_flag(Flag::Subtraction),false);
    assert_eq!(cpu.get_flag(Flag::Zero),false);
}
