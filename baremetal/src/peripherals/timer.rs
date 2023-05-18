use core::time::Duration;

use super::{utils::{MmioReg32, compile_time_size_assert, memory_barrier}, PERIPHERALS_BASE_ADDRESS};

const TIMER_BASE_ADDRESS:usize = PERIPHERALS_BASE_ADDRESS + 0x3000;

// timer frequency is 1_000_000 hz
// This number is based on this - https://www.youtube.com/watch?v=2dlBZoLCMSc

#[repr(C, align(4))]
struct TimerRegisters{
    _contorl_status:MmioReg32,
    counter_low:MmioReg32,
    counter_high:MmioReg32
}
compile_time_size_assert!(TimerRegisters, 0xC);

pub struct Timer{
    registers:&'static mut TimerRegisters,
    current_tick:u64
}

impl Timer{
    pub(super) fn new()->Self{
        let registers = unsafe{&mut *(TIMER_BASE_ADDRESS as *mut TimerRegisters)};
        let mut timer = Self { registers, current_tick:0 };
        // init current tick with valid value
        timer.current_tick = timer.get_timer_counter();
        return timer;
    }

    pub fn tick(&mut self)->Duration{
        let last_tick = self.current_tick;
        self.current_tick = self.get_timer_counter();
        let diff = self.current_tick - last_tick;
        // Since freq is 1_000_000 hz
        return Duration::from_micros(diff);
    }

    pub fn wait(&mut self, duration:Duration){
        let mut counter = core::time::Duration::ZERO;
        let _ = self.tick();  // reset the timer ticking and disard the result
        while counter < duration{
            let time_from_last_tick = self.tick();
            counter += time_from_last_tick;
        }
    }

    fn get_timer_counter(&self)->u64{
        memory_barrier();
        let low = self.registers.counter_low.read();
        let high = self.registers.counter_high.read();
        memory_barrier();
        
        return low as u64 | ((high as u64) << 32);
    }
}