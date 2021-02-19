use crate::{mmu::{io_ports::IoPorts, memory::UnprotectedMemory}, utils::memory_registers::DIV_REGISTER_ADDRESS};

use super::gb_timer::GbTimer;

pub fn update_timer_registers(timer:&mut GbTimer, memory:&mut IoPorts){
    let ports = memory.get_ports_cycle_trigger();
    if ports[0x04]{
        timer.system_counter = 0;
        ports[0x04] = false;
    }
}