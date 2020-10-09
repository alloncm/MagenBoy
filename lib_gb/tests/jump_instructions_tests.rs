mod memory_stub;

use lib_gbc::cpu::gb_cpu::GbCpu;
use lib_gbc::cpu::opcodes::jump_instructions::rst;
use crate::memory_stub::MemoryStub;

macro_rules! rst_test{
    ($name:ident, $opcode:expr, $expected_pc:expr) => {
        #[test]
        fn $name(){
            let mut cpu = GbCpu::default();
            cpu.stack_pointer =0xFFFE;
            let mut memory = MemoryStub{data:[0;0xFFFF]};

            rst(&mut cpu,&mut memory,$opcode);

            assert_eq!(cpu.program_counter, $expected_pc);
        }
    };
}

rst_test!(rst_c7_test,0xC7, 0x00);
rst_test!(rst_cf_test,0xCF, 0x08);
rst_test!(rst_d7_test,0xD7, 0x10);
rst_test!(rst_df_test,0xDF, 0x18);
rst_test!(rst_e7_test,0xE7, 0x20);
rst_test!(rst_ef_test,0xEF, 0x28);
rst_test!(rst_f7_test,0xF7, 0x30);
rst_test!(rst_ff_test,0xFF, 0x38);