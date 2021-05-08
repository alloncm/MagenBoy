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

pub fn get_div(timer: &GbTimer)->u8{
    (timer.system_counter >> 8) as u8 
}

pub fn set_tima(timer: &mut GbTimer, value:u8){
    timer.tima_register = value;
    timer.tima_overflow = false;
}

pub fn set_tma(timer: &mut GbTimer, value:u8){
    timer.tma_register = value;
}

pub fn set_tac(timer: &mut GbTimer, value:u8){
    timer.tac_tegister = value & 0b111;
}

//Reset on write
pub fn reset_div(timer: &mut GbTimer){
    timer.system_counter = 0;
}