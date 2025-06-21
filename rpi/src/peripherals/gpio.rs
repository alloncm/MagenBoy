#[cfg(feature = "bm")]
pub use no_std_impl::*;
#[cfg(feature = "os")]
pub use std_impl::*;


#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum Mode{
    Input   = 0b000,
    Output  = 0b001,
    Alt0    = 0b100,
    Alt3    = 0b111,
    Alt5    = 0b010,
}

pub enum GpioPull{
    None = 0,
    PullUp = 0b01,
}

pub enum Trigger{
    RisingEdge
}

#[cfg(feature = "bm")]
pub mod no_std_impl{
    use magenboy_common::synchronization::Mutex;

    use crate::peripherals::utils::{compile_time_size_assert, MmioReg32, get_static_peripheral, memory_barrier, BulkWrite};
    use super::*;
    
    #[repr(C,align(4))]
    struct GpioRegisters{
        gpfsel:[MmioReg32;6],
        _pad0:u32,
        gpset:[MmioReg32;2],
        _pad1:u32,
        gpclr:[MmioReg32;2],
        _pad2:u32,
        gplev:[MmioReg32;2],
        _pad3:u32,
        gpeds:[MmioReg32;2],
        _pad4:u32,
        gpren:[MmioReg32;2],
        _pad5:[u32;36],
        gpio_pup_pdn_cntrl:[MmioReg32;4]
    }
    compile_time_size_assert!(GpioRegisters, 0xF4);

    const GPIO_PINS_COUNT:usize = if cfg!(rpi = "4") {58} else {54};
    const BASE_GPIO_OFFSET: usize = 0x20_0000;
    static mut GPIO_REGISTERS:Option<Mutex<&'static mut GpioRegisters>> = None;

    pub struct GpioManager{
        pins_availability:[bool;GPIO_PINS_COUNT]
    }

    impl GpioManager{
        pub(in crate::peripherals) fn new()->GpioManager{
            unsafe{GPIO_REGISTERS = Some(Mutex::new(get_static_peripheral(BASE_GPIO_OFFSET)));}
            GpioManager { pins_availability: [true;GPIO_PINS_COUNT]}
        }

        pub fn take_pin(&mut self, bcm_pin_number:u8)->GpioPin{
            if self.pins_availability[bcm_pin_number as usize]{
                self.pins_availability[bcm_pin_number as usize] = false;
                return GpioPin::new(bcm_pin_number);
            }
            core::panic!("Pin {} is already taken", bcm_pin_number);
        }

        // This function is busy waiting for edge cases and not really polling interrupts.
        // when the interrupt controller will be implemented it could work
        pub fn poll_interrupts(&mut self,pins:&[InputGpioPin], reset_before_poll:bool){
            let gpio_registers = unsafe{GPIO_REGISTERS.as_mut().unwrap()};
            let pins_mask = pins
                .iter()
                .map(|p|p.inner.bcm_pin_number)
                .fold(0_u64, |value, bcm_number| {value | (1 << bcm_number)});
        
            memory_barrier();
            if reset_before_poll{
                gpio_registers.lock(|r|{
                    // reset the event detection
                    r.gpeds[0].write(0);
                    r.gpeds[1].write(0);
                });
            }
            log::info!("polling gpio joypad input...");
            loop{
                let registers_value:u64 = gpio_registers.lock(|r|r.gpeds[0].read() as u64 | (r.gpeds[1].read() as u64) << 32);
                let detected_pins = registers_value & pins_mask;
                if detected_pins != 0{
                    log::info!("Detected gpio input interrupt");
                    // reset the state of the registers
                    gpio_registers.lock(|r|{
                        r.gpeds[0].write(0xFFFF_FFFF);
                        r.gpeds[1].write(0xFFFF_FFFF);
                    });
                    memory_barrier();
                    return;
                }
            }
        }

        pub fn power_off(&mut self){
            memory_barrier();
            let registers = unsafe{GPIO_REGISTERS.as_mut().unwrap()};
            registers.lock(|r|{
                r.gpfsel.write(0);
                cfg_if::cfg_if!{ if #[cfg(rpi = "4")]{
                    r.gpio_pup_pdn_cntrl.write(0);  
                }
                else{compile_error!("Power off only support rpi4")}}
            });
            memory_barrier();
        }
    }

    pub struct GpioPin{
        registers:&'static Mutex<&'static mut GpioRegisters>,
        bcm_pin_number:u8,
    }

    impl GpioPin{
        pub(super) fn new(bcm_pin_number:u8)->Self{
            let registers = unsafe{GPIO_REGISTERS.as_mut().unwrap()};
            return Self{registers, bcm_pin_number};
        }

        pub fn into_input(mut self, pull:GpioPull)->InputGpioPin{
            self.set_mode(Mode::Input);
            self.set_pull(pull);
            return InputGpioPin { inner: self };
        }

        pub fn into_output(mut self)->OutputGpioPin{
            self.set_mode(Mode::Output);
            return OutputGpioPin { inner: self };
        }

        pub fn into_io(mut self, io_mode:Mode)->IoGpioPin{
            match io_mode{
                Mode::Alt0 |
                Mode::Alt3 |
                Mode::Alt5 => self.set_mode(io_mode),
                Mode::Input |
                Mode::Output => core::panic!("set mode param must be alt: {}", io_mode as u8)
            }
            return IoGpioPin { inner: self };
        }

        fn set_pull(&mut self, pull_mode:GpioPull){
            let register_index = self.bcm_pin_number / 16;
            let offset = (self.bcm_pin_number % 16) * 2;
            let mask:u32 = 0b11 << offset;
            memory_barrier();
            cfg_if::cfg_if!{ if #[cfg(rpi = "4")]{       
                let register_value = self.registers.lock(|r|r.gpio_pup_pdn_cntrl[register_index as usize].read());
                let new_value = (register_value & !mask) | ((pull_mode as u32) << offset as u32);
                self.registers.lock(|r|r.gpio_pup_pdn_cntrl[register_index as usize].write(new_value));
            }
            else{compile_error!("rpi's other than 4 needs proper support in order for set_pull to work")}}

            memory_barrier();
        }

        fn get_single_bit_registers_offset_and_mask(&self)->(u8, u32){
            let offset = self.bcm_pin_number / 32;          // since each registers contains 32 pins
            let mask = 1 << (self.bcm_pin_number % 32);    // get the position in the register
            return (offset, mask);
        }

        fn set_mode(&mut self, mode:Mode){
            let gpfsel_register = (self.bcm_pin_number / 10) as usize;            // each registers contains 10 pins
            let gpfsel_register_offset = (self.bcm_pin_number % 10) * 3;    // each pin take 3 bits in the register
            memory_barrier();
            self.registers.lock(|r|{
                let mut register_value = r.gpfsel[gpfsel_register].read();
                register_value &= !(0b111 << gpfsel_register_offset);   // clear the specific bits
                r.gpfsel[gpfsel_register].write(register_value | (mode as u32) << gpfsel_register_offset)
            });
            memory_barrier();
        }

        fn set_state(&mut self, state:bool){
            let (register, value) = self.get_single_bit_registers_offset_and_mask();
            memory_barrier();
            if state{
                self.registers.lock(|r|r.gpset[register as usize].write(value));
            }
            else{
                self.registers.lock(|r|r.gpclr[register as usize].write(value));
            }
            memory_barrier();
        }

        fn read_state(&self)->bool{
            let (gplev_register, mask) = self.get_single_bit_registers_offset_and_mask();
            memory_barrier();
            let value = self.registers.lock(|r|r.gplev[gplev_register as usize].read());
            memory_barrier();
            return value & mask != 0;
        }
    }


    pub struct InputGpioPin{
        inner: GpioPin
    }

    impl InputGpioPin{
        pub fn read_state(&self)->bool{self.inner.read_state()}

        pub fn set_interrupt(&mut self, trigger:Trigger){
            let (register_index, mask) = self.inner.get_single_bit_registers_offset_and_mask();
            memory_barrier();
            self.inner.registers.lock(|r|{
                match trigger{
                    Trigger::RisingEdge =>{
                        let register = &mut r.gpren[register_index as usize];
                        let value = register.read() | mask;
                        register.write(value);
                    }
                }
            });
            memory_barrier();
        }
    }

    impl Clone for InputGpioPin{
        fn clone(&self) -> Self {
            Self { 
                inner: GpioPin { 
                    registers: unsafe{GPIO_REGISTERS.as_mut().unwrap()}, 
                    bcm_pin_number: self.inner.bcm_pin_number 
                } 
            }
        }
    }

    pub struct OutputGpioPin{
        inner: GpioPin
    }

    impl OutputGpioPin{
        pub fn set_state(&mut self, state:bool){self.inner.set_state(state)}
    }

    pub struct IoGpioPin{
        inner: GpioPin
    }

    impl IoGpioPin{
        pub fn set_pull(&mut self, pull:GpioPull){
            self.inner.set_pull(pull);
        }
    }
}

#[cfg(feature = "os")]
pub mod std_impl{
    use rppal::gpio::{Gpio, InputPin, OutputPin, IoPin};
    use super::*;

    pub struct GpioManager{
        gpio:Gpio
    }

    impl GpioManager{
        pub(in crate::peripherals) fn new()->Self{
            Self { gpio: Gpio::new().unwrap() }
        }

        pub fn take_pin(&mut self, bcm_pin_number:u8)->GpioPin{
            return GpioPin { bcm_bumber: bcm_pin_number, gpio:self.gpio.clone()}
        }

        /// Blocks untill there is an interrupt
        pub fn poll_interrupts(&self, pins:&[InputGpioPin], reset_before_poll:bool){
            let pins:Vec<&InputPin> = pins.iter().map(|p|p.pin.as_ref()).collect();
            let (_pin, _level) = self.gpio.poll_interrupts(&pins, reset_before_poll, None).unwrap().unwrap();
        }
    }

    pub struct GpioPin{
        bcm_bumber:u8,
        gpio:Gpio
    }

    impl GpioPin{
        pub fn into_input(self, pull:GpioPull)->InputGpioPin{
            let pin = self.gpio.get(self.bcm_bumber).unwrap();
            let pin = match pull{
                GpioPull::None => pin.into_input(),
                GpioPull::PullUp => pin.into_input_pullup(),
            };
            return InputGpioPin { pin: std::sync::Arc::new(pin) }
        }
        pub fn into_output(self)->OutputGpioPin{OutputGpioPin { pin: self.gpio.get(self.bcm_bumber).unwrap().into_output() }}
        pub fn into_io(self, mode:Mode)->IoGpioPin{
            let pin = self.gpio.get(self.bcm_bumber).unwrap();
            let pin = match mode{
                Mode::Alt0 => pin.into_io(rppal::gpio::Mode::Alt0),
                Mode::Alt3 => pin.into_io(rppal::gpio::Mode::Alt3),
                Mode::Alt5 => pin.into_io(rppal::gpio::Mode::Alt5),
                Mode::Input |
                Mode::Output => std::panic!("Cant set io pin to input or output")
            };
            return IoGpioPin{pin};
        }
    }

    pub struct InputGpioPin{
        pin:std::sync::Arc<InputPin>
    }

    impl InputGpioPin{
        pub fn read_state(&self)->bool{self.pin.is_high()}

        pub fn set_interrupt(&mut self, t:Trigger){
            match t{
                Trigger::RisingEdge => std::sync::Arc::get_mut(&mut self.pin).unwrap().set_interrupt(rppal::gpio::Trigger::RisingEdge).unwrap(),
            }
        }
    }

    impl Clone for InputGpioPin{
        fn clone(&self) -> Self {
            Self { pin: self.pin.clone() }
        }
    }

    pub struct OutputGpioPin{
        pin:OutputPin
    }

    impl OutputGpioPin{
        pub fn set_state(&mut self, state:bool){self.pin.write(state.into())}
    }

    pub struct IoGpioPin{
        pin:IoPin
    }

    impl IoGpioPin{
        pub fn set_pull(&mut self, pull_mode:GpioPull){
            match pull_mode{
                GpioPull::None => self.pin.set_pullupdown(rppal::gpio::PullUpDown::Off),
                GpioPull::PullUp => todo!(),
            }
        }
    }
}


// Sugar syntax functions
impl OutputGpioPin{
    pub fn set_high(&mut self){
        self.set_state(true)
    }

    pub fn set_low(&mut self){
        self.set_state(false);
    }
}