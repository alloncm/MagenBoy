use crate::syncronization::Mutex;

use super::{PERIPHERALS_BASE_ADDRESS, utils::{MmioReg32, compile_time_size_assert, memory_barrier}};

#[repr(C,align(4))]
struct GpioRegisters{
    gpfsel:[MmioReg32;6],
    _pad0:u32,
    gpset:[MmioReg32;2],
    _pad1:u32,
    gpclr:[MmioReg32;2],
    _pad2:u32,
    pglev:[MmioReg32;2],
    _pad3:[u32;42],
    gpio_pup_pdn_cntrl:[MmioReg32;4]
}
compile_time_size_assert!(GpioRegisters, 0xF4);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum Mode{
    Input = 0,
    Output = 1,
    Alt0 = 4,
    Alt5 = 2
}

pub enum GpioPull{
    None = 0,
    _PullUp = 0b01,
}


const RPI4_GPIO_PINS_COUNT:usize = 58;
const BASE_GPIO_ADDRESS: usize = PERIPHERALS_BASE_ADDRESS + 0x20_0000;
static mut GPIO_REGISTERS:Option<Mutex<&'static mut GpioRegisters>> = None;

pub struct GpioManager{
    pins_availability:[bool;RPI4_GPIO_PINS_COUNT]
}

impl GpioManager{
    pub(super) fn new()->GpioManager{
        unsafe{GPIO_REGISTERS = Some(Mutex::new(&mut *(BASE_GPIO_ADDRESS as *mut GpioRegisters)));}
        GpioManager { pins_availability: [true;RPI4_GPIO_PINS_COUNT]}
    }

    pub fn take_pin(&mut self, bcm_pin_number:u8, mode:Mode)->GpioPin{
        if self.pins_availability[bcm_pin_number as usize]{
            self.pins_availability[bcm_pin_number as usize] = false;
            return GpioPin::new(bcm_pin_number, mode);
        }
        core::panic!("Pin {} is already taken", bcm_pin_number);
    }
}

pub struct GpioPin{
    mode:Mode,
    registers:&'static mut Mutex<&'static mut GpioRegisters>,
    bcm_pin_number:u8,
}

impl GpioPin{
    pub(super) fn new(bcm_pin_number:u8, mode:Mode)->Self{
        let registers = unsafe{GPIO_REGISTERS.as_mut().unwrap()};
        let mut pin = Self{mode:Mode::Input, registers, bcm_pin_number};
        pin.set_mode(mode);
        return pin;
    }

    pub fn set_mode(&mut self, mode:Mode){
        let gpfsel_register = (self.bcm_pin_number / 10) as usize;            // each registers contains 10 pins
        let gpfsel_register_offset = (self.bcm_pin_number % 10) * 3;    // each pin take 3 bits in the register
        memory_barrier();
        self.registers.lock(|r|{
            let mut register_value = r.gpfsel[gpfsel_register].read();
            register_value &= !(0b111 << gpfsel_register_offset);   // clear the specific bits
            r.gpfsel[gpfsel_register].write(register_value | (mode as u32) << gpfsel_register_offset)
        });
        memory_barrier();
        self.mode = mode;
    }

    pub fn set_state(&mut self, state:bool){
        debug_assert!(self.mode == Mode::Output);
        let register = self.bcm_pin_number / 32;      // since each registers contains 32 pins
        let value = 1 << (self.bcm_pin_number % 32);       // get the position in the register
        memory_barrier();
        if state{
            self.registers.lock(|r|r.gpset[register as usize].write(value));
        }
        else{
            self.registers.lock(|r|r.gpclr[register as usize].write(value));
        }
        memory_barrier();
    }

    pub fn read_state(&self)->bool{
        debug_assert!(self.mode == Mode::Input);
        
        let gplev_register = self.bcm_pin_number / 32;  // since each registers contains 32 pins
        let mask = 1 << (self.bcm_pin_number % 32);       // get the position in the register
        memory_barrier();
        let value = self.registers.lock(|r|r.pglev[gplev_register as usize].read());
        memory_barrier();
        return value & mask != 0;
    }

    pub fn set_pull(&mut self, pull_mode:GpioPull){
        let register_index = self.bcm_pin_number / 16;
        let offset = (self.bcm_pin_number % 16) * 2;
        let mask:u32 = 0b11 << offset;
        memory_barrier();
        let register_value = self.registers.lock(|r|r.gpio_pup_pdn_cntrl[register_index as usize].read());
        let new_value = (register_value & !mask) | ((pull_mode as u32) << offset as u32);
        self.registers.lock(|r|r.gpio_pup_pdn_cntrl[register_index as usize].write(new_value));
        memory_barrier();
    }

    // Sugar syntax functions
    pub fn set_high(&mut self){
        self.set_state(true)
    }

    pub fn set_low(&mut self){
        self.set_state(false);
    }
}