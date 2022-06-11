use lib_gb::keypad::{joypad::{Joypad, NUM_OF_KEYS},joypad_provider::JoypadProvider,button::Button};
use lib_gb::utils::create_array;
use rppal::gpio::{Gpio, InputPin};

pub type GpioPin = u8;

pub struct GpioJoypadProvider{
    input_pins:[InputPin;NUM_OF_KEYS]
}

impl GpioJoypadProvider{
    pub fn new<F:Fn(&Button)->GpioPin>(mapper:F)->Self{   
        let gpio = Gpio::new().unwrap();
        let buttons = [Button::A,Button::B,Button::Start,Button::Select,Button::Up,Button::Down,Button::Right,Button::Left];
        let mut counter = 0;
        let generator_lambda = ||{
            let button = &buttons[counter];
            let result = gpio.get(mapper(button)).unwrap().into_input();
            counter += 1;
            return result;
        };
        let pins:[InputPin;NUM_OF_KEYS] = create_array(generator_lambda);

        return Self{input_pins:pins};
    }
}

impl JoypadProvider for GpioJoypadProvider{
    fn provide(&mut self, joypad:&mut Joypad){
        for i in 0..NUM_OF_KEYS{
            joypad.buttons[i] = self.input_pins[i].is_high();
        }
    }
}