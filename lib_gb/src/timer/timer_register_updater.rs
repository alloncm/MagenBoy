use crate::{mmu::io_ports::{IoPorts, DIV_REGISTER_INDEX, IO_PORTS_MEMORY_OFFSET}, utils::memory_registers::TIMA_REGISTER_ADDRESS};
use super::gb_timer::GbTimer;

const TIMA_REGISTER_INDEX: usize = (TIMA_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;

pub fn update_timer_registers(timer:&mut GbTimer, memory:&mut IoPorts){
    let ports = memory.get_ports_cycle_trigger();
    if ports[DIV_REGISTER_INDEX as usize]{
        timer.system_counter = 0;
        ports[DIV_REGISTER_INDEX as usize] = false;
    }
    if ports[TIMA_REGISTER_INDEX]{
        timer.tima_overflow = false;
        ports[TIMA_REGISTER_INDEX] = false;
    }
}