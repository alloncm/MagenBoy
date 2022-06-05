use rppal::gpio::{OutputPin, IoPin};

use super::{ili9341_controller::{Ili9341Commands, SPI_BUFFER_SIZE, SpiController}};

// This will work only if spi is enabled at /boot/config.txt (dtparam=spi=on)
pub struct RppalSpi{
    spi_device:rppal::spi::Spi,
    dc_pin:OutputPin
}

impl RppalSpi{
    const CLOCK_SPEED:u32 = 75_000_000;         // chose based on trial and error
    const SPI_TRANSFER_MAX_SIZE:usize = 4096;

    fn write<const SIZE:usize>(&mut self, command:Ili9341Commands, data:&[u8;SIZE]){
        let command = command as u8;
        self.dc_pin.set_low();
        self.spi_device.write(&[command]).expect("Error while writing to the spi device");
        self.dc_pin.set_high();
        let chunks = data.chunks(Self::SPI_TRANSFER_MAX_SIZE);
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
            Self::CLOCK_SPEED, 
            rppal::spi::Mode::Mode0
        ).expect("Error creating rppal spi device");

        let dc_pin = rppal::gpio::Gpio::new().unwrap().get(dc_pin).unwrap().into_output();
        return RppalSpi { spi_device, dc_pin };
    }

    fn write<const SIZE:usize>(&mut self, command:Ili9341Commands, data:&[u8;SIZE]) {
        self.write(command, data);
    }

    fn write_buffer(&mut self, command:Ili9341Commands, data:&[u8;SPI_BUFFER_SIZE]) {
        self.write(command, data);
    }
}