use libc::c_void;
use rppal::gpio::{OutputPin, IoPin};

use crate::rpi_gpio::{dma::DmaTransferer, libc_abort};

use super::{ili9341_controller::{Ili9341Commands, SpiController, TARGET_SCREEN_HEIGHT, TARGET_SCREEN_WIDTH}, decl_write_volatile_field, decl_read_volatile_field};


const BCM2835_GPIO_BASE_ADDRESS:usize = 0x20_0000;
const BCM2835_SPI0_BASE_ADDRESS:usize = 0x20_4000;
const BCM2835_RPI4_BUS_ADDRESS:usize = 0xFE00_0000;
const BCM_RPI4_MMIO_PERIPHERALS_SIZE:usize = 0x180_0000;

pub struct Bcm2835{
    ptr:*mut c_void,
    mem_fd: libc::c_int
}

impl Bcm2835 {
    fn new()->Self{
        let mem_fd = unsafe{libc::open(std::ffi::CStr::from_bytes_with_nul(b"/dev/mem\0").unwrap().as_ptr(), libc::O_RDWR | libc::O_SYNC)};
        
        if mem_fd < 0{
            libc_abort("bad file descriptor");
        }
        
        let bcm2835 = unsafe{libc::mmap(
            std::ptr::null_mut(), 
            BCM_RPI4_MMIO_PERIPHERALS_SIZE,
            libc::PROT_READ | libc::PROT_WRITE, 
            libc::MAP_SHARED, 
            mem_fd,
            BCM2835_RPI4_BUS_ADDRESS as libc::off_t
        )};

        if bcm2835 == libc::MAP_FAILED{
            libc_abort("FATAL: mapping /dev/mem failed!");
        }

        Bcm2835 { ptr: bcm2835, mem_fd }
    }

    pub fn get_ptr(&self, offset:usize)->*mut c_void{
        unsafe{self.ptr.add(offset)}
    }

    pub fn get_fd(&self)->libc::c_int{
        self.mem_fd
    }
}

impl Drop for Bcm2835{
    fn drop(&mut self) {
        unsafe{
            let result = libc::munmap(self.ptr, BCM_RPI4_MMIO_PERIPHERALS_SIZE);
            if result != 0{
                libc_abort("Error while unmapping the mmio memory");
            }

            let result = libc::close(self.mem_fd);
            if result != 0{
                libc_abort("Error while closing the mem_fd");
            }
        }
    }
}

struct GpioRegistersAccess{
    ptr:*mut u32
}
enum GpioRegister{
    Gpfsel0 = 0,
    Gpfsel1 = 1,
    Gpset0 = 6,
    Gpset1 = 7,
    Gpclr0 = 8,
    Gpclr1 = 9
}
impl GpioRegistersAccess{
    unsafe fn read_register(&self, register:GpioRegister)->u32{
        std::ptr::read_volatile(self.ptr.add(register as usize))
    }
    unsafe fn write_register(&self, register:GpioRegister, value:u32){
        std::ptr::write_volatile(self.ptr.add(register as usize), value);
    }
    unsafe fn set_gpio_mode(&self, pin:u8, mode:u8){
        // there are less than 100 pins so I assume the largest one is less than 100
        let gpfsel_register = pin / 10;
        let gpfsel_register_index = pin % 10;
        let register_ptr = self.ptr.add(gpfsel_register as usize);
        let mut register_value = std::ptr::read_volatile(register_ptr);
        let mask = !(0b111 << (gpfsel_register_index * 3));
        register_value &= mask;
        register_value |= (mode as u32) << (gpfsel_register_index *3);
        std::ptr::write_volatile(register_ptr, register_value);
    }
    unsafe fn set_gpio_high(&self, pin:u8){
        if pin < 32{
            std::ptr::write_volatile(self.ptr.add(GpioRegister::Gpset0 as usize), 1 << pin);
        }
        else{
            std::ptr::write_volatile(self.ptr.add(GpioRegister::Gpset1 as usize), 1 << (pin - 32));
        }
    }
    unsafe fn set_gpio_low(&self, pin:u8){
        if pin < 32{
            std::ptr::write_volatile(self.ptr.add(GpioRegister::Gpclr0 as usize), 1 << pin);
        }
        else{
            std::ptr::write_volatile(self.ptr.add(GpioRegister::Gpclr1 as usize), 1 << (pin - 32));
        }
    }
}

// The register are 4 bytes each so making sure the allignment and padding are correct
#[repr(C, align(4))]
struct SpiRegistersAccess{
    control_status:u32,
    fifo:u32,
    clock:u32,
    data_length:u32
}

impl SpiRegistersAccess{
    decl_write_volatile_field!(write_cs, control_status);
    decl_read_volatile_field!(read_cs, control_status);
    decl_write_volatile_field!(write_fifo, fifo);
    decl_write_volatile_field!(write_clk, clock);
    decl_write_volatile_field!(write_dlen, data_length);
}

pub struct RawSpi{
    spi_registers: *mut SpiRegistersAccess,
    spi_pins:[IoPin;2],
    spi_cs0:OutputPin,
    dc_pin:OutputPin,

    dma_transferer:DmaTransferer<{Self::DMA_SPI_CHUNK_SIZE}, {Self::DMA_SPI_NUM_CHUNKS}>,
    last_transfer_was_dma:bool,

    // declared last in order for it to be freed last 
    // rust gurantee that the order of the droped values is the order of declaration
    // keeping it last so it will be freed correctly
    _bcm2835:Bcm2835,
}

impl RawSpi{
    const SPI_CS_RXF:u32 = 1 << 20;
    const SPI_CS_RXR:u32 = 1 << 19;
    const SPI_CS_TXD:u32 = 1 << 18;
    const SPI_CS_DONE:u32 = 1 << 16;
    const SPI_CS_DMAEN:u32 = 1 << 8;
    const SPI_CS_TA:u32 = 1 << 7;
    const SPI_CS_CLEAR:u32 = 3<<4;
    const SPI_CS_CLEAR_RX:u32 = 1 << 5;

    fn new (dc_pin:OutputPin, spi_pins:[IoPin;2], mut spi_cs0: OutputPin)->Self{
        let bcm2835 = Bcm2835::new();

        // let gpio_registers = unsafe{GpioRegistersAccess{ptr:bcm2835.add(Self::BCM2835_GPIO_BASE_ADDRESS) as *mut u32}};
        let spi_registers = bcm2835.get_ptr(BCM2835_SPI0_BASE_ADDRESS) as *mut SpiRegistersAccess;

        unsafe{
            // ChipSelect = 0, ClockPhase = 0, ClockPolarity = 0
            spi_cs0.set_low();
            (*spi_registers).write_cs(Self::SPI_CS_TA);
            (*spi_registers).write_clk(4);
            (*spi_registers).write_dlen(2);
        }

        log::info!("finish ili9341 device init");

        let dma_transferer = DmaTransferer::new(&bcm2835, 7, 1 );
        RawSpi { 
            _bcm2835:bcm2835, spi_registers, dc_pin, spi_pins, spi_cs0, last_transfer_was_dma: false,
            dma_transferer
        }
    }

    fn write<const SIZE:usize>(&mut self, command:Ili9341Commands, data:&[u8;SIZE]){
        self.prepare_for_transfer();
        self.dc_pin.set_low();
        self.write_raw(&[command as u8]);
        self.dc_pin.set_high();
        self.write_raw(data);
        self.last_transfer_was_dma = false;
    }

    fn prepare_for_transfer(&mut self) {
        if self.last_transfer_was_dma{
            self.dma_transferer.end_dma_transfer();
            unsafe{
                (*self.spi_registers).write_cs(Self::SPI_CS_TA | Self::SPI_CS_CLEAR);
                (*self.spi_registers).write_dlen(2);        // poll mode speed up
            }
        }
    }

    fn write_raw<const SIZE:usize>(&mut self, data:&[u8;SIZE]){
        unsafe{
            let mut current_index = 0;
            while current_index < SIZE {
                let cs:u32 = (*self.spi_registers).read_cs();
                if cs & Self::SPI_CS_TXD != 0{
                    (*self.spi_registers).write_fifo(data[current_index] as u32);
                    current_index += 1;
                }
                if (cs & (Self::SPI_CS_RXR | Self::SPI_CS_RXF)) != 0 {
                    (*self.spi_registers).write_cs(Self::SPI_CS_TA | Self::SPI_CS_CLEAR_RX);
                }
            }

            // wait for the last trasfer to finish
            while ((*self.spi_registers).read_cs() & Self::SPI_CS_DONE) == 0 {
                if ((*self.spi_registers).read_cs() & (Self::SPI_CS_RXR | Self::SPI_CS_RXF)) != 0{
                    (*self.spi_registers).write_cs(Self::SPI_CS_TA | Self::SPI_CS_CLEAR_RX);
                }
            }
        }
    }

    const MAX_DMA_SPI_TRANSFER:usize = 0xFFE0;  // must be smaller than max u16 and better be alligned for 32 bytes
    const DMA_SPI_TRANSFER_SIZE:usize = TARGET_SCREEN_HEIGHT * TARGET_SCREEN_WIDTH * std::mem::size_of::<u16>();
    const DMA_SPI_NUM_CHUNKS:usize = (Self::DMA_SPI_TRANSFER_SIZE / Self::MAX_DMA_SPI_TRANSFER) + ((Self::DMA_SPI_TRANSFER_SIZE % Self::MAX_DMA_SPI_TRANSFER) != 0) as usize;
    const DMA_SPI_CHUNK_SIZE:usize = (Self::DMA_SPI_TRANSFER_SIZE / Self::DMA_SPI_NUM_CHUNKS) + 4;
    const DMA_TI_PERMAP_SPI_TX:u8 = 6;
    const DMA_TI_PERMAP_SPI_RX:u8 = 7;
    const DMA_SPI_FIFO_PHYS_ADDRESS:u32 = 0x7E20_4004;

    fn write_dma<const SIZE:usize>(&mut self, command:Ili9341Commands, data:&[u8;SIZE]){
        self.prepare_for_transfer();
        
        self.dc_pin.set_low();
        self.write_raw(&[command as u8]);
        self.dc_pin.set_high();
        self.write_dma_raw(&data);
        self.last_transfer_was_dma = true;
    }


    // Since generic_const_exprs is not stable yet Im reserving the first 4 bytes of the data variable for internal use
    fn write_dma_raw<const SIZE:usize>(&mut self, data:&[u8;SIZE]){
        unsafe{
            (*self.spi_registers).write_cs(Self::SPI_CS_DMAEN | Self::SPI_CS_CLEAR);
            let data_len = Self::DMA_SPI_CHUNK_SIZE - 4;  // Removing the first 4 bytes from this length param
            let header = [Self::SPI_CS_TA as u8, 0, (data_len & 0xFF) as u8,  /*making sure this is little endian order*/ (data_len >> 8) as u8];

            let chunks = data.chunks_exact(Self::DMA_SPI_CHUNK_SIZE - 4);
            let mut array:[u8;Self::DMA_SPI_CHUNK_SIZE * Self::DMA_SPI_NUM_CHUNKS] = [0;Self::DMA_SPI_CHUNK_SIZE * Self::DMA_SPI_NUM_CHUNKS];
            let mut i = 0;
            for chunk in chunks{
                std::ptr::copy_nonoverlapping(header.as_ptr(), array.as_mut_ptr().add(i * Self::DMA_SPI_CHUNK_SIZE), 4);
                std::ptr::copy_nonoverlapping(chunk.as_ptr(), array.as_mut_ptr().add(4 + (i * Self::DMA_SPI_CHUNK_SIZE)), Self::DMA_SPI_CHUNK_SIZE - 4);
                i += 1;
            }

            self.dma_transferer.start_dma_transfer(&array, 
                Self::DMA_TI_PERMAP_SPI_TX,
                Self::DMA_SPI_FIFO_PHYS_ADDRESS, 
                Self::DMA_TI_PERMAP_SPI_RX, 
                Self::DMA_SPI_FIFO_PHYS_ADDRESS
            );
        }
    }
}

impl SpiController for RawSpi{
    fn new(dc_pin_number:u8)->Self {
        let gpio = rppal::gpio::Gpio::new().unwrap();
        let spi0_ceo_n = gpio.get(8).unwrap().into_output();
        let spi0_mosi = gpio.get(10).unwrap().into_io(rppal::gpio::Mode::Alt0);
        let spi0_sclk = gpio.get(11).unwrap().into_io(rppal::gpio::Mode::Alt0);
        let dc_pin = gpio.get(dc_pin_number).unwrap().into_output();

        RawSpi::new(dc_pin, [spi0_mosi, spi0_sclk], spi0_ceo_n)
    }

    fn write<const SIZE:usize>(&mut self, command:Ili9341Commands, data:&[u8;SIZE]) {
        self.write(command, data);
    }

    fn write_buffer(&mut self, command:Ili9341Commands, data:&[u8;TARGET_SCREEN_HEIGHT * TARGET_SCREEN_WIDTH * 2]) {
        self.write_dma(command, data);
    }
}