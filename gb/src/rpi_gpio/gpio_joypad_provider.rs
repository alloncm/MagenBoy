use lib_gb::keypad::{joypad::{Joypad, NUM_OF_KEYS},joypad_provider::JoypadProvider,button::Button};
use lib_gb::utils::create_array;
use rppal::gpio::{Gpio, InputPin};

use crate::joypad_menu::MenuJoypadProvider;

pub type GpioBcmPin = u8;

static mut INPUT_PINS:Option<[InputPin; NUM_OF_KEYS]> = None;

pub struct GpioJoypadProvider{
    input_pins:&'static [InputPin; NUM_OF_KEYS]
}

impl GpioJoypadProvider{
    // FIXME: The mapper is used only the first this struct is initialized
    pub fn new<F:Fn(&Button)->GpioBcmPin>(mapper:F)->Self{
        if unsafe{INPUT_PINS.is_none()}{
            let gpio = Gpio::new().unwrap();
            let mut counter:u8 = 0;
            let generator_lambda = ||{
                let button:Button = unsafe{std::mem::transmute(counter)};
                let mut result = gpio.get(mapper(&button)).unwrap().into_input();
                result.set_interrupt(rppal::gpio::Trigger::RisingEdge).unwrap();
                counter += 1;
                return result;
            };
            unsafe{INPUT_PINS = Some(create_array(generator_lambda))};
        }
        let input_pins:&'static [InputPin; NUM_OF_KEYS] = unsafe{INPUT_PINS.as_ref().unwrap()};
        return Self{input_pins};
    }
}

impl JoypadProvider for GpioJoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad){
        for i in 0..NUM_OF_KEYS{
            joypad.buttons[i] = self.input_pins[i].is_high();
        }
    }
}

impl MenuJoypadProvider for GpioJoypadProvider{
    fn poll(&mut self, mut joypad:&mut Joypad) {
        let pins_refs:[&InputPin; NUM_OF_KEYS] = {
            let pins = self.input_pins;
            // Replace this line with each_ref() once it become stable
            [&pins[0],&pins[1],&pins[2],&pins[3],&pins[4],&pins[5],&pins[6], &pins[7]]
        };
        let _ = rppal::gpio::Gpio::new().unwrap().poll_interrupts( &pins_refs, true, None).unwrap().unwrap();
        self.provide(&mut joypad);
    }
}