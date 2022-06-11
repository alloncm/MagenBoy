use std::ops::Add;

use lib_gb::ppu::{gb_ppu::{SCREEN_WIDTH, SCREEN_HEIGHT}, gfx_device::{GfxDevice, Pixel}};
use rppal::gpio::OutputPin;

pub enum Ili9341Commands{
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

const ILI9341_SCREEN_WIDTH:usize = 320;
const ILI9341_SCREEN_HEIGHT:usize = 240;
const SCALE:f32 = 5.0 / 3.0;    // maximum scale to fit the ili9341 screen
pub (super) const TARGET_SCREEN_WIDTH:usize = (SCREEN_WIDTH as f32 * SCALE) as usize;
pub (super) const TARGET_SCREEN_HEIGHT:usize = (SCREEN_HEIGHT as f32 * SCALE) as usize;
const FRAME_BUFFER_X_OFFSET:usize = (ILI9341_SCREEN_WIDTH - TARGET_SCREEN_WIDTH) / 2;

pub const SPI_BUFFER_SIZE:usize = TARGET_SCREEN_HEIGHT * TARGET_SCREEN_WIDTH * std::mem::size_of::<u16>();

pub trait SpiController {
    fn new(dc_pin_number:u8)->Self;
    fn write<const SIZE:usize>(&mut self, command:Ili9341Commands, data:&[u8;SIZE]);
    fn write_buffer(&mut self, command:Ili9341Commands, data:&[u8;SPI_BUFFER_SIZE]);
}

struct Ili9341Contoller<SC:SpiController>{
    spi:SC,
    led_pin: OutputPin,
    reset_pin: OutputPin
}

impl<SC:SpiController> Ili9341Contoller<SC>{
    const CLEAN_BUFFER:[u8;ILI9341_SCREEN_HEIGHT * ILI9341_SCREEN_WIDTH * std::mem::size_of::<u16>()] = [0; ILI9341_SCREEN_HEIGHT * ILI9341_SCREEN_WIDTH * std::mem::size_of::<u16>()];

    pub fn new(reset_pin_bcm:u8, dc_pin_bcm:u8, led_pin_bcm:u8)->Self{

        log::info!("Initalizing with screen size width: {}, hight: {}", TARGET_SCREEN_WIDTH, TARGET_SCREEN_HEIGHT);

        let gpio = rppal::gpio::Gpio::new().unwrap();
        let mut reset_pin = gpio.get(reset_pin_bcm).unwrap().into_output();
        let mut led_pin = gpio.get(led_pin_bcm).unwrap().into_output();

        // toggling the reset pin to initalize the lcd
        reset_pin.set_high();
        Self::sleep_ms(120);
        reset_pin.set_low();
        Self::sleep_ms(120);
        reset_pin.set_high();
        Self::sleep_ms(120);

        let mut spi:SC = SpiController::new(dc_pin_bcm);

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
        spi.write(Ili9341Commands::ColumnAddressSet, &[0,0,((ILI9341_SCREEN_WIDTH -1) >> 8) as u8, ((ILI9341_SCREEN_WIDTH -1) & 0xFF) as u8]);
        spi.write(Ili9341Commands::PageAddressSet, &[0,0,((ILI9341_SCREEN_HEIGHT -1) >> 8) as u8, ((ILI9341_SCREEN_HEIGHT -1) & 0xFF) as u8]);
        // using write and not write buffer since this is not the correct size 
        spi.write(Ili9341Commands::MemoryWrite, &Self::CLEAN_BUFFER);

        // turn backlight on
        led_pin.set_high();

        log::info!("finish ili9341 device init");

        return Ili9341Contoller { spi, led_pin, reset_pin};
    }


    pub fn write_frame_buffer(&mut self, buffer:&[u16;SCREEN_HEIGHT*SCREEN_WIDTH]){
        let mut scaled_buffer: [u8;TARGET_SCREEN_HEIGHT * TARGET_SCREEN_WIDTH * 2] = [0;TARGET_SCREEN_HEIGHT * TARGET_SCREEN_WIDTH * 2];
        unsafe{image_inter::scale_biliniear_c::<SCREEN_WIDTH, SCREEN_HEIGHT, TARGET_SCREEN_WIDTH, TARGET_SCREEN_HEIGHT>(buffer.as_ptr(), scaled_buffer.as_mut_ptr())};

        let end_x_index = TARGET_SCREEN_WIDTH + FRAME_BUFFER_X_OFFSET - 1;
        self.spi.write(Ili9341Commands::ColumnAddressSet, &[
            (FRAME_BUFFER_X_OFFSET >> 8) as u8,
            (FRAME_BUFFER_X_OFFSET & 0xFF) as u8, 
            (end_x_index >> 8) as u8, 
            (end_x_index & 0xFF) as u8 
        ]);
        self.spi.write(Ili9341Commands::PageAddressSet, &[
            0x0, 0x0,
            ((TARGET_SCREEN_HEIGHT - 1) >> 8) as u8, 
            ((TARGET_SCREEN_HEIGHT - 1) & 0xFF) as u8 
        ]);

        self.spi.write_buffer(Ili9341Commands::MemoryWrite, &scaled_buffer);
    }

    fn sleep_ms(milliseconds_to_sleep:u64){
        std::thread::sleep(std::time::Duration::from_millis(milliseconds_to_sleep));
    }
}

impl<SC:SpiController> Drop for Ili9341Contoller<SC>{
    fn drop(&mut self) {
        self.led_pin.set_low();
        self.reset_pin.set_high();
        Self::sleep_ms(1);
        self.reset_pin.set_low();
    }
}

pub struct Ili9341GfxDevice<SC:SpiController>{
    ili9341_controller:Ili9341Contoller<SC>,
    turbo_mul:u8,
    turbo_frame_counter:u8,

    frame_limiter:u32,
    frames_counter: u32,
    time_counter:std::time::Duration,
    last_time: std::time::Instant,
}

impl<SC:SpiController> Ili9341GfxDevice<SC>{
    pub fn new(reset_pin_bcm:u8, dc_pin_bcm:u8, led_pin_bcm:u8, turbo_mul:u8, frame_limiter:u32)->Self{
        #[cfg(not(feature = "u16pixel"))]
        std::compile_error("ili9341 gfx device must have Pixel type = u16");

        let ili9341_controller = Ili9341Contoller::new(reset_pin_bcm, dc_pin_bcm, led_pin_bcm);

        Ili9341GfxDevice {
            ili9341_controller,frames_counter:0,
            time_counter: std::time::Duration::ZERO, last_time:std::time::Instant::now(),
            turbo_mul, turbo_frame_counter:0, frame_limiter
        }
    }
}

impl<SC:SpiController> GfxDevice for Ili9341GfxDevice<SC>{
    fn swap_buffer(&mut self, buffer:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        self.turbo_frame_counter = (self.turbo_frame_counter + 1) % self.turbo_mul;
        if self.turbo_frame_counter != 0{
            return;
        }

        if self.frames_counter & self.frame_limiter == 0{
            self.ili9341_controller.write_frame_buffer(&buffer);
        }

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