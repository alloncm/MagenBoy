use rppal::gpio::{OutputPin, IoPin};

use super::{ili9341_controller::{Ili9341Commands, TARGET_SCREEN_WIDTH, TARGET_SCREEN_HEIGHT, SpiController}};

pub struct RppalSpi{
    spi_device:rppal::spi::Spi,
    dc_pin:OutputPin
}

impl RppalSpi{
    fn write<const SIZE:usize>(&mut self, command:Ili9341Commands, data:&[u8;SIZE]){
        let command = command as u8;
        self.dc_pin.set_low();
        self.spi_device.write(&[command]).expect("Error while writing to the spi device");
        self.dc_pin.set_high();
        let chunks = data.chunks(4096);
        for chunk in chunks{
            self.spi_device.write(&chunk).expect(std::format!("Error wrting data to spi device for command: {:#X}",command).as_str() );
        }
    }
}

impl SpiController for RppalSpi{
    fn new(dc_pin:u8)->Self{
        let spi_device = rppal::spi::Spi::new(
            rppal::spi::Bus::Spi0,
            rppal::spi::SlaveSelect::Ss0/*pin 24*/, 
            75_000_000/*In order to be able to achieve higher fps*/, 
            rppal::spi::Mode::Mode0
        ).expect("Error creating rppal spi device");

        let dc_pin = rppal::gpio::Gpio::new().unwrap().get(dc_pin).unwrap().into_output();
        return RppalSpi { spi_device, dc_pin };
    }

    fn write<const SIZE:usize>(&mut self, command:Ili9341Commands, data:&[u8;SIZE]) {
        self.write(command, data);
    }

    fn write_buffer(&mut self, command:Ili9341Commands, data:&[u8;TARGET_SCREEN_HEIGHT * TARGET_SCREEN_WIDTH * 2]) {
        self.write(command, data);
    }
}