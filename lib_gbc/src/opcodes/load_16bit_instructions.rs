use crate::cpu::gbc_cpu::GbcCpu;

//load into 16bit register RR the value NN
pub fn load_rr_nn(cpu:&mut GbcCpu, register_index:u8, value:u16){
    cpu.set_16bit_register(register_index, value);
}

//loads register HL into the SP
pub fn load_sp_hl(cpu:&mut GbcCpu){
    cpu.stack_pointer = cpu.hl();
}