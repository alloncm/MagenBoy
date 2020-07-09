extern crate lib_gbc;
use lib_gbc::cpu::gbc_cpu::{GbcCpu};
use lib_gbc::opcodes::load_16bit_instructions;
use lib_gbc::mmu::memory::Memory;

struct MemoryStub{
    pub data:[u8;0xFFFF]
}

impl Memory for MemoryStub{
    fn read(&self, address:u16)->u8{
        self.data[address as usize]
    }

    fn write(&mut self, address:u16, value:u8){
        self.data[address as usize] = value;
    }
}

#[test]
fn test_ld_hl_sp_dd(){
    let opcode:u16 = 0x23;
    let mut cpu = GbcCpu::default();
    load_16bit_instructions::ld_hl_spdd(&mut cpu, opcode);
    assert_eq!(*cpu.hl.value(), opcode);
}

#[test]
fn test_ld_rr_nn(){
    let opcode:u32 = 0x31FEFF;
    let mut cpu = GbcCpu::default();
    load_16bit_instructions::load_rr_nn(&mut cpu, opcode);
    assert_eq!(cpu.stack_pointer, 0xFFFE);
}

#[test]
fn test_push_af(){
    //arrange
    //PUSH AF opcode
    let opcode = 0xF5;
    let mut cpu = GbcCpu::default();
    *cpu.af.value() = 0xEFC7;
    cpu.stack_pointer = 0xFFFE;		
    let mut mmu = MemoryStub{
        data:[0;0xFFFF]
    };

    //Act
    load_16bit_instructions::push(&mut cpu, &mut mmu, opcode);

    //Assert
    assert_eq!(cpu.stack_pointer, 0xFFFC);
    assert_eq!(*cpu.af.high(), mmu.data[0xFFFD]);
    assert_eq!(*cpu.af.low(), mmu.data[0xFFFC]);
}

#[test]
fn test_pop_af(){
    //arrange
    //PUSH AF opcode
    let opcode = 0xF1;
    let mut cpu = GbcCpu::default();
    cpu.stack_pointer = 0xFFFC;		
    let mut mmu = MemoryStub{
        data:[0;0xFFFF]
    };
    mmu.data[0xFFFC] = 0x54;
    mmu.data[0xFFFD] = 0x98;

    *cpu.af.value() = 0;

    //Act
    load_16bit_instructions::pop(&mut cpu, &mut mmu, opcode);

    //Assert
    assert_eq!(cpu.stack_pointer, 0xFFFE);
    assert_eq!(*cpu.af.value(), 0x9850);
}