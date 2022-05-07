use std::ops::Add;

use lib_gb::ppu::{gfx_device::GfxDevice, gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}};
use rppal::gpio::OutputPin;


pub struct Ili9341GfxDevice{
    ili9341_controller:Ili9341Contoller,
    frames_counter: u32,
    time_counter:std::time::Duration,
    last_time: std::time::Instant,
}

impl Ili9341GfxDevice{
    pub fn new()->Self{
        let ili9341_controller = Ili9341Contoller::new();
        Ili9341GfxDevice {ili9341_controller,frames_counter:0, time_counter: std::time::Duration::ZERO, last_time:std::time::Instant::now()}
    }
}

impl GfxDevice for Ili9341GfxDevice{
    fn swap_buffer(&mut self, buffer:&[u32; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        let u16_buffer:[u16;SCREEN_HEIGHT*SCREEN_WIDTH] = buffer.map(|pixel| {
            let b = pixel & 0xFF;
            let g = (pixel & 0xFF00)>>8;
            let r = (pixel & 0xFF0000)>>16; 
            let mut u16_pixel = b as u16 >> 3;
            u16_pixel |= ((g >> 2) << 5) as u16;
            u16_pixel |= ((r >> 3) << 11) as u16;
            return u16_pixel;
        });
        self.ili9341_controller.write_frame_buffer(&u16_buffer);

        // measure fps
        self.frames_counter += 1;
        let time = std::time::Instant::now();
        self.time_counter = self.time_counter.add(time.duration_since(self.last_time));
        self.last_time = time;
        if self.time_counter.as_millis() > 1000{
            log::info!("FPS: {}", self.frames_counter);
            self.frames_counter = 0;
            self.time_counter = std::time::Duration::ZERO;
        }
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
    
    fn write<const SIZE:usize>(&mut self, command: Ili9341Commands, data:&[u8; SIZE]) {
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

    const ILI9341_SCREEN_WIDTH:usize = 320;
    const ILI9341_SCREEN_HEIGHT:usize = 240;
    const SCALE:f32 = 5.0 / 3.0;    // maximum scale to fit the ili9341 screen
    const TARGET_SCREEN_WIDTH:usize = (SCREEN_WIDTH as f32 * Self::SCALE) as usize;
    const TARGET_SCREEN_HEIGHT:usize = (SCREEN_HEIGHT as f32 * Self::SCALE) as usize;
    const FRAME_BUFFER_X_OFFSET:usize = (Self::ILI9341_SCREEN_WIDTH - Self::TARGET_SCREEN_WIDTH) / 2;

    const CLEAN_BUFFER:[u8;Self::ILI9341_SCREEN_HEIGHT * Self::ILI9341_SCREEN_WIDTH * 2] = [0; Self::ILI9341_SCREEN_HEIGHT * Self::ILI9341_SCREEN_WIDTH * 2];

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
        /*---------------------------------------------------------------------------------------------------------------------- */
        // Reset the screen
        spi.write(Ili9341Commands::SoftwareReset,&[]);
        Self::sleep_ms(5);
        spi.write(Ili9341Commands::DisplayOff,&[]);

        // Some power stuff, probably uneccessary but just for sure
        spi.write(Ili9341Commands::PowerControlA, &[0x39, 0x2C, 0x0, 0x34, 0x2]);
        spi.write(Ili9341Commands::PowerControlB, &[0x0, 0xC1, 0x30]);
        spi.write(Ili9341Commands::DriverTimingControlA, &[0x85, 0x0, 0x78]);
        spi.write(Ili9341Commands::DriverTimingControlB, &[0x0, 0x0]);
        spi.write(Ili9341Commands::PowerOnSequenceControl, &[0x64, 0x3, 0x12, 0x81]);
        spi.write(Ili9341Commands::PowerControl1, &[0x23]);
        spi.write(Ili9341Commands::PowerControl2,&[0x10]);
        spi.write(Ili9341Commands::VcomControl1, &[0xE3, 0x28]);
        spi.write(Ili9341Commands::VcomControl2, &[0x86]);

        // Configuring the screen
        spi.write(Ili9341Commands::MemoryAccessControl, &[0x20]); // This command tlit the screen 90 degree
        spi.write(Ili9341Commands::PixelFormatSet, &[0x55]);     // set pixel format to 16 bit per pixel;
        spi.write(Ili9341Commands::FrameRateControl, &[0x0, 0x1F /*According to the docs this is 61 hrz */]);
        spi.write(Ili9341Commands::DisplayFunctionControl, &[0x8, 0x82, 0x27]);
        
        // Gamma values - pretty sure its redundant
        spi.write(Ili9341Commands::Enable3G, &[0x2]);
        spi.write(Ili9341Commands::GammaSet, &[0x1]);
        spi.write(Ili9341Commands::PossitiveGammaCorrection,&[0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1, 0x37, 0x07, 0x10, 0x03, 0x0E, 0x09, 0x00]);     
        spi.write(Ili9341Commands::NegativeGammaCorrection, &[0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1, 0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36, 0x0F]);

        // Turn screen on
        spi.write(Ili9341Commands::SleepOut,&[]);
        Self::sleep_ms(120);
        spi.write(Ili9341Commands::DisplayOn,&[]);
        /*---------------------------------------------------------------------------------------------------------------------- */
        //End of fbcp-ili9341 inpired implementation

        // Clear screen
        spi.write(Ili9341Commands::ColumnAddressSet, &[0,0,((Self::ILI9341_SCREEN_WIDTH -1) >> 8) as u8, ((Self::ILI9341_SCREEN_WIDTH -1) & 0xFF) as u8]);
        spi.write(Ili9341Commands::PageAddressSet, &[0,0,((Self::ILI9341_SCREEN_HEIGHT -1) >> 8) as u8, ((Self::ILI9341_SCREEN_HEIGHT -1) & 0xFF) as u8]);
        spi.write(Ili9341Commands::MemoryWrite, &Self::CLEAN_BUFFER);

        // turn backlight on
        led_pin.set_high();

        log::info!("Initalizing with screen size width: {}, hight: {}", Self::TARGET_SCREEN_WIDTH, Self::TARGET_SCREEN_HEIGHT);

        return Ili9341Contoller { spi, led_pin, reset_pin};
    }


    pub fn write_frame_buffer(&mut self, buffer:&[u16;SCREEN_HEIGHT*SCREEN_WIDTH]){
        let end_x_index = Self::TARGET_SCREEN_WIDTH + Self::FRAME_BUFFER_X_OFFSET - 1;
        self.spi.write(Ili9341Commands::ColumnAddressSet, &[
            (Self::FRAME_BUFFER_X_OFFSET >> 8) as u8,
            (Self::FRAME_BUFFER_X_OFFSET & 0xFF) as u8, 
            (end_x_index >> 8) as u8, 
            (end_x_index & 0xFF) as u8 
        ]);
        self.spi.write(Ili9341Commands::PageAddressSet, &[
            0x0, 0x0,
            ((Self::TARGET_SCREEN_HEIGHT - 1) >> 8) as u8, 
            ((Self::TARGET_SCREEN_HEIGHT - 1) & 0xFF) as u8 
        ]);
        
        let mut scaled_buffer: [u16;Self::TARGET_SCREEN_HEIGHT * Self::TARGET_SCREEN_WIDTH] = [0;Self::TARGET_SCREEN_HEIGHT * Self::TARGET_SCREEN_WIDTH];
        Self::scale_to_screen(buffer, &mut scaled_buffer);
        let mut u8_buffer:[u8;Self::TARGET_SCREEN_HEIGHT*Self::TARGET_SCREEN_WIDTH*2] = [0;Self::TARGET_SCREEN_HEIGHT*Self::TARGET_SCREEN_WIDTH*2];
        for i in 0..scaled_buffer.len(){
            u8_buffer[i*2] = (scaled_buffer[i] >> 8) as u8;
            u8_buffer[(i*2)+1] = (scaled_buffer[i] & 0xFF) as u8;
        }

        self.spi.write(Ili9341Commands::MemoryWrite, &u8_buffer);
    }


    // This function implements bilinear interpolation scaling according to this article - http://tech-algorithm.com/articles/bilinear-image-scaling/
    fn scale_to_screen(input_buffer:&[u16;SCREEN_HEIGHT*SCREEN_WIDTH], output_buffer:&mut [u16;Self::TARGET_SCREEN_HEIGHT*Self::TARGET_SCREEN_WIDTH]){
        // not sure why the -1.0
        let x_ratio = (SCREEN_WIDTH as f32 - 1.0) / Self::TARGET_SCREEN_WIDTH as f32;
        let y_ratio = (SCREEN_HEIGHT as f32 - 1.0) / Self::TARGET_SCREEN_HEIGHT as f32;

        let mut offset_counter = 0;
        for y in 0..Self::TARGET_SCREEN_HEIGHT{
            for x in 0..Self::TARGET_SCREEN_WIDTH{
                let x_val = (x_ratio * x as f32) as u32;            // x value of a point in this ratio between 0 and x
                let y_val = (y_ratio * y as f32) as u32;            // y value of a point in this ratio between o and y
                let x_diff = (x_ratio * x as f32) - x_val as f32;   
                let y_diff = (y_ratio * y as f32) - y_val as f32;
                let original_pixel_index = (y_val as usize * SCREEN_WIDTH) + x_val as usize;

                // Get the pixel and 3 surounding pixels
                let pixel_a = input_buffer[original_pixel_index];
                let pixel_b = input_buffer[original_pixel_index + 1];
                let pixel_c = input_buffer[original_pixel_index + SCREEN_WIDTH];
                let pixel_d = input_buffer[original_pixel_index + SCREEN_WIDTH + 1];

                let blue:f32 = ((pixel_a & 0x1F) as f32 * (1.0-x_diff) * (1.0-y_diff)) + 
                               ((pixel_b & 0x1F) as f32 * (x_diff)*(1.0-y_diff)) + 
                               ((pixel_c & 0x1F) as f32 * y_diff * (1.0-x_diff)) + 
                               ((pixel_d & 0x1F) as f32 * x_diff * y_diff);
                let green:f32 = (((pixel_a >> 5) & 0x3F) as f32 * (1.0-x_diff) * (1.0-y_diff)) + 
                                (((pixel_b >> 5) & 0x3F) as f32 * (x_diff)*(1.0-y_diff)) + 
                                (((pixel_c >> 5) & 0x3F) as f32 * y_diff * (1.0-x_diff)) + 
                                (((pixel_d >> 5) & 0x3F) as f32 * x_diff * y_diff);
                let red:f32 = (((pixel_a >> 11) & 0x1F) as f32 * (1.0-x_diff) * (1.0-y_diff)) + 
                              (((pixel_b >> 11) & 0x1F) as f32 * (x_diff)*(1.0-y_diff)) + 
                              (((pixel_c >> 11) & 0x1F) as f32 * y_diff * (1.0-x_diff)) + 
                              (((pixel_d >> 11) & 0x1F) as f32 * x_diff * y_diff);

                output_buffer[offset_counter] = blue as u16 | ((green as u16) << 5) | ((red as u16) << 11);
                offset_counter += 1;
            }
        }
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