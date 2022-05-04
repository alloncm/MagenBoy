use lib_gb::ppu::{gfx_device::GfxDevice, gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}};
use rppal::gpio::OutputPin;

pub struct Ili9341GfxDevice{
    ili9341_controller:Ili9341Contoller
}

impl Ili9341GfxDevice{
    pub fn new()->Self{
        let ili9341_controller = Ili9341Contoller::new();
        Ili9341GfxDevice {ili9341_controller}
    }
}

impl GfxDevice for Ili9341GfxDevice{
    fn swap_buffer(&mut self, buffer:&[u32; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        let u16_buffer:[u16;SCREEN_HEIGHT*SCREEN_WIDTH] = buffer.map(|pixel| {
            let r = pixel & 0xFF;
            let g = (pixel & 0xFF00)>>8;
            let b = (pixel & 0xFF0000)>>16;
            let mut u16_pixel = r as u16 >> 3;
            u16_pixel |= ((g >> 2) << 5) as u16;
            u16_pixel |= ((b >> 3) << 11) as u16;
            return u16_pixel;
        });
        self.ili9341_controller.write_frame_buffer(&u16_buffer);
    }
}

struct RppalSpi{
    spi_device:rppal::spi::Spi,
    dc_pin:OutputPin
}

impl RppalSpi{
    pub fn new(dc_pin:OutputPin)->Self{
        let spi_device = rppal::spi::Spi::new(
            rppal::spi::Bus::Spi0,
            rppal::spi::SlaveSelect::Ss0/*pin 24*/, 
            75_000_000/*In order to be able to achieve 60fps*/, 
            rppal::spi::Mode::Mode0
        ).expect("Error creating rppal spi device");

        return RppalSpi { spi_device, dc_pin };
    }
    
    fn write<const SIZE:usize>(&mut self, command: Ili9341Commands, data:[u8; SIZE]) {
        let error = "Error while writing to the spi device";
        let command = command as u8;
        self.dc_pin.set_low();
        self.spi_device.write(&[command]).expect(error);
        self.dc_pin.set_high();
        let chunks = data.chunks(4096);
        for chunk in chunks{
            self.spi_device.write(&chunk).expect(std::format!("Error wrting data to spi device for command: {:#X}",command).as_str() );
        }
    }
}

enum Ili9341Commands{
    SoftwareReset = 0x01,
    SleepOut = 0x11,
    GammaSet = 0x26,
    DisplayOff = 0x28,
    DisplayOn = 0x29,
    ColumnAddressSet = 0x2A,            // Set curosr X value
    PageAddressSet = 0x2B,              // Set cursor Y value
    MemoryWrite = 0x2C,
    MemoryAccessControl = 0x36,
    PixelFormatSet = 0x3A,
    FrameRateControl = 0xB1,
    DisplayFunctionControl = 0xB6,
    PowerControl1 = 0xC0,
    PowerControl2 = 0xC1,
    VcomControl1 = 0xC5,
    VcomControl2 = 0xC7,
    PowerControlA = 0xCB,
    PowerControlB = 0xCF,
    PossitiveGammaCorrection = 0xE0,
    NegativeGammaCorrection = 0xE1,
    DriverTimingControlA = 0xE8,
    DriverTimingControlB = 0xEA,
    PowerOnSequenceControl = 0xED,      
    Enable3G = 0xF2,
}

struct Ili9341Contoller{
    spi:RppalSpi,
    led_pin: OutputPin,
    reset_pin: OutputPin
}

impl Ili9341Contoller{
    pub fn new()->Self{
        let gpio = rppal::gpio::Gpio::new().unwrap();
        let mut dc_pin = gpio.get(15).unwrap().into_output();
        let mut reset_pin = gpio.get(14).unwrap().into_output();
        let mut led_pin = gpio.get(25).unwrap().into_output();


        // toggling the reset pin to initalize the lcd
        let wait_duration = std::time::Duration::from_millis(120);
        reset_pin.set_high();
        std::thread::sleep(wait_duration);
        reset_pin.set_low();
        std::thread::sleep(wait_duration);
        reset_pin.set_high();
        std::thread::sleep(wait_duration);

        let mut spi: RppalSpi = RppalSpi::new(dc_pin);

        // This code snippets is ofcourse wrriten by me but took heavy insperation from fbcp-ili9341 (https://github.com/juj/fbcp-ili9341)
        // I used the ili9341 application notes and the fbcp-ili9341 implementation in order to write it all down
        // And later I twicked some params specific to my display (http://www.lcdwiki.com/3.2inch_SPI_Module_ILI9341_SKU:MSP3218)

        // There is another implementation in rust for an ili9341 controller which is much simpler and uses those commands:
        // Sleepms(5), SoftwareReset, Sleepms(120), MemoryAccessControl, PixelFormatSet, SleepOut, Sleepms(5), DisplayOn
        // minimal config based on rust ili9341 lib (https://github.com/yuri91/ili9341-rs)

        // fbcp-ili9341 inspired implementation:
        /*--------------------------------------------------------------------------- */
        // Reset the screen
        spi.write(Ili9341Commands::SoftwareReset,[]);
        Self::sleep_ms(5);
        spi.write(Ili9341Commands::DisplayOff,[]);

        // Some power stuff, probably uneccessary but just for sure
        spi.write(Ili9341Commands::PowerControlA, [0x39, 0x2C, 0x0, 0x34, 0x2]);
        spi.write(Ili9341Commands::PowerControlB, [0x0, 0xC1, 0x30]);
        spi.write(Ili9341Commands::DriverTimingControlA, [0x85, 0x0, 0x78]);
        spi.write(Ili9341Commands::DriverTimingControlB, [0x0, 0x0]);
        spi.write(Ili9341Commands::PowerOnSequenceControl, [0x64, 0x3, 0x12, 0x81]);
        spi.write(Ili9341Commands::PowerControl1, [0x23]);
        spi.write(Ili9341Commands::PowerControl2,[0x10]);
        spi.write(Ili9341Commands::VcomControl1, [0xE3, 0x28]);
        spi.write(Ili9341Commands::VcomControl2, [0x86]);

        // Configuring the screen
        spi.write(Ili9341Commands::MemoryAccessControl, [0x20]); // this command can affect various graphics options like RGB format and screen fip
        spi.write(Ili9341Commands::PixelFormatSet, [0x55]);     // set pixel format to 16 bit per pixel;
        spi.write(Ili9341Commands::FrameRateControl, [0x0, 0x1F /*According to the docs this is 61 hrz */]);
        spi.write(Ili9341Commands::DisplayFunctionControl, [0x8, 0x82, 0x27]);
        
        // Gamma values - pretty sure its redundant
        spi.write(Ili9341Commands::Enable3G, [0x2]);
        spi.write(Ili9341Commands::GammaSet, [0x1]);
        spi.write(Ili9341Commands::PossitiveGammaCorrection,[0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1, 0x37, 0x07, 0x10, 0x03, 0x0E, 0x09, 0x00]);     
        spi.write(Ili9341Commands::NegativeGammaCorrection, [0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1, 0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36, 0x0F]);

        // Turn screen on
        spi.write(Ili9341Commands::SleepOut,[]);
        Self::sleep_ms(120);
        spi.write(Ili9341Commands::DisplayOn,[]);
        /*------------------------------------------------------------------------------- */
        //End of fbcp-ili9341 inpired implementation

        // turn backlight on
        led_pin.set_high();

        return Ili9341Contoller { spi, led_pin, reset_pin};
    }

    pub fn write_frame_buffer(&mut self, buffer:&[u16;SCREEN_HEIGHT*SCREEN_WIDTH]){
        self.spi.write(Ili9341Commands::ColumnAddressSet, [0x0,0x0, ((SCREEN_WIDTH - 1) >> 8) as u8, ((SCREEN_WIDTH - 1) & 0xFF) as u8 ]);
        self.spi.write(Ili9341Commands::PageAddressSet, [0x0,0x0, ((SCREEN_HEIGHT - 1) >> 8) as u8, ((SCREEN_HEIGHT - 1) & 0xFF) as u8 ]);
        // let u8_buffer:&[u8; SCREEN_HEIGHT*SCREEN_WIDTH*2] = unsafe{
        //     let ptr = (buffer as *const u16) as *const u8;
        //     &*(ptr as *const [u8; SCREEN_HEIGHT*SCREEN_WIDTH*2])
        // };
        let mut u8_buffer:[u8;SCREEN_HEIGHT*SCREEN_WIDTH*2] = [0;SCREEN_HEIGHT*SCREEN_WIDTH*2];
        for i in 0..buffer.len(){
            u8_buffer[i*2] = (buffer[i] >> 8) as u8;
            u8_buffer[(i*2)+1] = (buffer[i] & 0xFF) as u8;
        }
        self.spi.write(Ili9341Commands::MemoryWrite, u8_buffer);
    }

    fn sleep_ms(milliseconds_to_sleep:u64){
        std::thread::sleep(std::time::Duration::from_millis(milliseconds_to_sleep));
    }
}

impl Drop for Ili9341Contoller{
    fn drop(&mut self) {
        self.led_pin.set_low();
        self.reset_pin.set_high();
        Self::sleep_ms(1);
        self.reset_pin.set_low();
    }
}