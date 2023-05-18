use lib_gb::ppu::{gb_ppu::{SCREEN_WIDTH, SCREEN_HEIGHT}, gfx_device::{GfxDevice, Pixel}};

use crate::peripherals::{GpioPin, Mode, Timer, Spi0, PERIPHERALS};

const ILI9341_SCREEN_WIDTH:usize = 320;
const ILI9341_SCREEN_HEIGHT:usize = 240;
const SCALE:f32 = 5.0 / 3.0;    // maximum scale to fit the ili9341 screen
pub(super) const TARGET_SCREEN_WIDTH:usize = (SCREEN_WIDTH as f32 * SCALE) as usize;
pub(super) const TARGET_SCREEN_HEIGHT:usize = (SCREEN_HEIGHT as f32 * SCALE) as usize;
const FRAME_BUFFER_X_OFFSET:usize = (ILI9341_SCREEN_WIDTH - TARGET_SCREEN_WIDTH) / 2;

pub const SPI_BUFFER_SIZE:usize = TARGET_SCREEN_HEIGHT * TARGET_SCREEN_WIDTH * core::mem::size_of::<u16>();

#[repr(u8)]
enum Ili9341Command{
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
    spi:Spi0,
    timer: Timer,
    led_pin: GpioPin,
    reset_pin: GpioPin
}

impl Ili9341Contoller{
    const CLEAN_BUFFER:[u8;ILI9341_SCREEN_HEIGHT * ILI9341_SCREEN_WIDTH * core::mem::size_of::<u16>()] = [0; ILI9341_SCREEN_HEIGHT * ILI9341_SCREEN_WIDTH * core::mem::size_of::<u16>()];

    pub fn new(reset_pin_bcm:u8, led_pin_bcm:u8)->Self{
        log::info!("Initalizing with screen size width: {}, hight: {}", TARGET_SCREEN_WIDTH, TARGET_SCREEN_HEIGHT);

        let gpio = unsafe{PERIPHERALS.get_gpio()};
        let reset_pin = gpio.take_pin(reset_pin_bcm, Mode::Output);
        let led_pin = gpio.take_pin(led_pin_bcm, Mode::Output);
        let spi = unsafe{PERIPHERALS.take_spi0()};

        let mut controller = Ili9341Contoller { spi, led_pin, reset_pin, timer: unsafe{PERIPHERALS.take_timer()}};

        // toggling the reset pin to initalize the lcd
        controller.reset_pin.set_high();
        controller.sleep_ms(120);
        controller.reset_pin.set_low();
        controller.sleep_ms(120);
        controller.reset_pin.set_high();
        controller.sleep_ms(120);


        // This code snippets is ofcourse wrriten by me but took heavy insperation from fbcp-ili9341 (https://github.com/juj/fbcp-ili9341)
        // I used the ili9341 application notes and the fbcp-ili9341 implementation in order to write it all down
        // And later I twicked some params specific to my display (http://www.lcdwiki.com/3.2inch_SPI_Module_ILI9341_SKU:MSP3218)

        // There is another implementation in rust for an ili9341 controller which is much simpler and uses those commands:
        // Sleepms(5), SoftwareReset, Sleepms(120), MemoryAccessControl, PixelFormatSet, SleepOut, Sleepms(5), DisplayOn
        // minimal config based on rust ili9341 lib (https://github.com/yuri91/ili9341-rs)

        // fbcp-ili9341 inspired implementation:
        /*---------------------------------------------------------------------------------------------------------------------- */
        // Reset the screen
        controller.spi.write(Ili9341Command::SoftwareReset as u8,&[]);
        controller.sleep_ms(5);
        controller.spi.write(Ili9341Command::DisplayOff as u8,&[]);

        // Some power stuff, probably uneccessary but just for sure
        controller.spi.write(Ili9341Command::PowerControlA as u8, &[0x39, 0x2C, 0x0, 0x34, 0x2]);
        controller.spi.write(Ili9341Command::PowerControlB as u8, &[0x0, 0xC1, 0x30]);
        controller.spi.write(Ili9341Command::DriverTimingControlA as u8, &[0x85, 0x0, 0x78]);
        controller.spi.write(Ili9341Command::DriverTimingControlB as u8, &[0x0, 0x0]);
        controller.spi.write(Ili9341Command::PowerOnSequenceControl as u8, &[0x64, 0x3, 0x12, 0x81]);
        controller.spi.write(Ili9341Command::PowerControl1 as u8, &[0x23]);
        controller.spi.write(Ili9341Command::PowerControl2 as u8,&[0x10]);
        controller.spi.write(Ili9341Command::VcomControl1 as u8, &[0xE3, 0x28]);
        controller.spi.write(Ili9341Command::VcomControl2 as u8, &[0x86]);

        // Configuring the screen
        controller.spi.write(Ili9341Command::MemoryAccessControl as u8, &[0x28]); // This command tlit the screen 90 degree and set pixel to BGR order 
        controller.spi.write(Ili9341Command::PixelFormatSet as u8, &[0x55]);     // set pixel format to 16 bit per pixel;
        controller.spi.write(Ili9341Command::FrameRateControl as u8, &[0x0, 0x10 /*According to the docs this is 119 hrz, setting this option in order to avoid screen tearing on rpi zero2 */]);
        controller.spi.write(Ili9341Command::DisplayFunctionControl as u8, &[0x8, 0x82, 0x27]);
        
        // Gamma values - pretty sure its redundant
        controller.spi.write(Ili9341Command::Enable3G as u8, &[0x2]);
        controller.spi.write(Ili9341Command::GammaSet as u8, &[0x1]);
        controller.spi.write(Ili9341Command::PossitiveGammaCorrection as u8,&[0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1, 0x37, 0x07, 0x10, 0x03, 0x0E, 0x09, 0x00]);     
        controller.spi.write(Ili9341Command::NegativeGammaCorrection as u8, &[0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1, 0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36, 0x0F]);

        // Turn screen on
        controller.spi.write(Ili9341Command::SleepOut as u8,&[]);
        controller.sleep_ms(120);
        controller.spi.write(Ili9341Command::DisplayOn as u8,&[]);
        /*---------------------------------------------------------------------------------------------------------------------- */
        //End of fbcp-ili9341 inpired implementation

        log::info!("Finish configuring ili9341");

        // turn backlight on
        controller.led_pin.set_high();

        // Clear screen
        controller.spi.write(Ili9341Command::ColumnAddressSet as u8, &[0,0,((ILI9341_SCREEN_WIDTH -1) >> 8) as u8, ((ILI9341_SCREEN_WIDTH -1) & 0xFF) as u8]);
        controller.spi.write(Ili9341Command::PageAddressSet as u8, &[0,0,((ILI9341_SCREEN_HEIGHT -1) >> 8) as u8, ((ILI9341_SCREEN_HEIGHT -1) & 0xFF) as u8]);
        // using write and not write buffer since this is not the correct size 
        controller.spi.write(Ili9341Command::MemoryWrite as u8, &Self::CLEAN_BUFFER);

        // need to sleep before changing the clock after transferring pixels to the lcd
        controller.sleep_ms(120);

        controller.spi.fast_mode();

        log::info!("finish ili9341 device init");

        return controller;
    }


    pub fn write_frame_buffer(&mut self, buffer:&[u16;SCREEN_HEIGHT*SCREEN_WIDTH]){
        let mut scaled_buffer: [u8;TARGET_SCREEN_HEIGHT * TARGET_SCREEN_WIDTH * 2] = [0;TARGET_SCREEN_HEIGHT * TARGET_SCREEN_WIDTH * 2];
        unsafe{image_inter::scale_bilinear::<SCREEN_WIDTH, SCREEN_HEIGHT, TARGET_SCREEN_WIDTH, TARGET_SCREEN_HEIGHT>(buffer.as_ptr(), scaled_buffer.as_mut_ptr())};

        let end_x_index = TARGET_SCREEN_WIDTH + FRAME_BUFFER_X_OFFSET - 1;
        self.spi.write(Ili9341Command::ColumnAddressSet as u8, &[
            (FRAME_BUFFER_X_OFFSET >> 8) as u8,
            (FRAME_BUFFER_X_OFFSET & 0xFF) as u8, 
            (end_x_index >> 8) as u8, 
            (end_x_index & 0xFF) as u8 
        ]);
        self.spi.write(Ili9341Command::PageAddressSet as u8, &[
            0x0, 0x0,
            ((TARGET_SCREEN_HEIGHT - 1) >> 8) as u8, 
            ((TARGET_SCREEN_HEIGHT - 1) & 0xFF) as u8 
        ]);

        self.spi.write_dma(Ili9341Command::MemoryWrite as u8, &scaled_buffer);
    }

    fn sleep_ms(&mut self, milliseconds_to_sleep:u64){
        let target_wait_time = core::time::Duration::from_millis(milliseconds_to_sleep);
        self.timer.wait(target_wait_time);
    }
}

impl Drop for Ili9341Contoller{
    fn drop(&mut self) {
        self.spi.write(Ili9341Command::DisplayOff as u8, &[]);
        self.led_pin.set_low();
        self.reset_pin.set_high();
        self.sleep_ms(1);
        self.reset_pin.set_low();
    }
}

pub struct Ili9341GfxDevice{
    ili9341_controller:Ili9341Contoller,
    turbo_mul:u8,
    turbo_frame_counter:u8,

    frame_limiter:u32,
    frames_counter: u32,
    time_counter:core::time::Duration,
}

impl Ili9341GfxDevice{
    pub fn new(reset_pin_bcm:u8, led_pin_bcm:u8, turbo_mul:u8, frame_limiter:u32)->Self{
        let mut ili9341_controller = Ili9341Contoller::new(reset_pin_bcm, led_pin_bcm);
        // reset the timer
        let _ = ili9341_controller.timer.tick();

        Ili9341GfxDevice {
            ili9341_controller,frames_counter:0,
            time_counter: core::time::Duration::ZERO,
            turbo_mul, turbo_frame_counter:0, frame_limiter
        }
    }
}


const EXPECTED_FRAME_DURATION: f64 = 1.0f64/60.0f64;
impl GfxDevice for Ili9341GfxDevice{
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
        let mut duration = self.ili9341_controller.timer.tick().as_secs_f64();

        // block for the frame duration
        while duration < EXPECTED_FRAME_DURATION{
            duration += self.ili9341_controller.timer.tick().as_secs_f64();
        }
        
        self.time_counter += core::time::Duration::from_secs_f64(duration);
        if self.time_counter.as_millis() > 1000{
            log::debug!("FPS: {}", self.frames_counter);
            self.frames_counter = 0;
            self.time_counter = core::time::Duration::ZERO;
        }
    }
}