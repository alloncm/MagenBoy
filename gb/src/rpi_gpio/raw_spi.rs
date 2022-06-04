use bcm_host::BcmHost;
use rppal::gpio::{OutputPin, IoPin};

use crate::rpi_gpio::{dma::DmaTransferer};

use super::{ili9341_controller::{Ili9341Commands, SpiController, TARGET_SCREEN_HEIGHT, TARGET_SCREEN_WIDTH}, decl_write_volatile_field, decl_read_volatile_field};

const BCM_SPI0_BASE_ADDRESS:usize = 0x20_4000;
const SPI_CLOCK_DIVISOR:u32 = 4;    // the smaller the faster (on my system below 4 there are currptions)

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
    dc_pin:OutputPin,
    dma_transferer:DmaTransferer<{Self::DMA_SPI_CHUNK_SIZE}, {Self::DMA_SPI_NUM_CHUNKS}>,
    last_transfer_was_dma:bool,
    
    // holding those pins in order to make sure they are configured correctly
    // the state resets upon drop
    _spi_pins:[IoPin;2], 
    _spi_cs0:OutputPin,

    // declared last in order for it to be freed last 
    // rust gurantee that the order of the droped values is the order of declaration
    // keeping it last so it will be freed correctly
    _bcm:BcmHost,
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
        let bcm_host = BcmHost::new();

        let spi_registers = bcm_host.get_ptr(BCM_SPI0_BASE_ADDRESS) as *mut SpiRegistersAccess;

        unsafe{
            // ChipSelect = 0, ClockPhase = 0, ClockPolarity = 0
            spi_cs0.set_low();
            Self::setup_poll_fast_transfer(&mut *spi_registers);
            (*spi_registers).write_clk(SPI_CLOCK_DIVISOR);
        }

        log::info!("finish ili9341 device init");

        let dma_transferer = DmaTransferer::new(&bcm_host, 7, 1 );
        RawSpi { 
            _bcm:bcm_host, spi_registers, dc_pin, _spi_pins: spi_pins, _spi_cs0: spi_cs0, last_transfer_was_dma: false,
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
            unsafe{Self::setup_poll_fast_transfer(&mut *self.spi_registers)};
        }
    }

    fn setup_poll_fast_transfer(spi_registers:&mut SpiRegistersAccess){
        unsafe{
            spi_registers.write_cs(Self::SPI_CS_TA | Self::SPI_CS_CLEAR);
            // poll mode speed up according to this forum post - https://forums.raspberrypi.com/viewtopic.php?f=44&t=181154
            spi_registers.write_dlen(2);        
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