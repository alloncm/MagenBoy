use crate::{peripherals::{Mode, dma::DmaSpiTransferer, utils::{memory_barrier, get_static_peripheral}, PERIPHERALS}, drivers, configuration::peripherals::*};

use super::{utils::{MmioReg32, compile_time_size_assert}, IoGpioPin, OutputGpioPin};

const SPI0_BASE_OFFSET:usize = 0x20_4000;

// Fix this usafe of the driver module here
pub(super)const SPI_BUFFER_SIZE:usize = drivers::SPI_BUFFER_SIZE;

// The register are 4 bytes each so making sure the allignment and padding are correct
#[repr(C, align(4))]
struct SpiRegisters{
    control_status:MmioReg32,
    fifo:MmioReg32,
    clock:MmioReg32,
    data_len:MmioReg32
}
compile_time_size_assert!(SpiRegisters, 0x10);

pub struct Spi0{
    spi_registers: &'static mut SpiRegisters,
    dc_pin:OutputGpioPin,
    last_transfer_was_dma:bool,
    dma:DmaSpiTransferer,
    
    // holding those pins in order to make sure they are configured correctly
    // the state resets upon drop
    _spi_pins:[IoGpioPin;2], 
    _spi_cs0:OutputGpioPin,
}

impl Spi0{
    const SPI0_CE0_N_BCM_PIN:u8 = 8;
    const SPI0_MOSI_BCM_PIN:u8 = 10;
    const SPI0_SCLK_BCM_PIN:u8 = 11;

    const SPI_CS_RXF:u32 = 1 << 20;
    const SPI_CS_RXR:u32 = 1 << 19;
    const SPI_CS_TXD:u32 = 1 << 18;
    const SPI_CS_DONE:u32 = 1 << 16;
    const SPI_CS_DMAEN:u32 = 1 << 8;
    const SPI_CS_TA:u32 = 1 << 7;
    const SPI_CS_CLEAR:u32 = 3<<4;
    const SPI_CS_CLEAR_RX:u32 = 1 << 5;

    pub(super) fn new (dc_pin:u8)->Self{
        let gpio = unsafe{PERIPHERALS.get_gpio()};
        let mut spi_cs0 = gpio.take_pin(Self::SPI0_CE0_N_BCM_PIN).into_output();
        let dc_pin = gpio.take_pin(dc_pin).into_output();
        let spi_pins = [
            gpio.take_pin(Self::SPI0_MOSI_BCM_PIN).into_io(Mode::Alt0), 
            gpio.take_pin(Self::SPI0_SCLK_BCM_PIN).into_io(Mode::Alt0)
        ];

        let spi_registers = get_static_peripheral(SPI0_BASE_OFFSET);

        // ChipSelect = 0, ClockPhase = 0, ClockPolarity = 0
        spi_cs0.set_low();
        Self::setup_poll_fast_transfer(&mut *spi_registers);
        spi_registers.clock.write(INIT_SPI_CLOCK_DIVISOR);

        memory_barrier();   // Change SPI to DMA
        let dma_transferer = DmaSpiTransferer::new(Self::SPI_CS_DMAEN);
        memory_barrier();   // Change DMA to SPI

        log::info!("Finish initializing spi mmio interface");
        Spi0 { 
            spi_registers, dc_pin, _spi_pins: spi_pins, _spi_cs0: spi_cs0, last_transfer_was_dma: false,
            dma:dma_transferer
        }
    }

    pub fn write<const SIZE:usize>(&mut self, command:u8, data:&[u8;SIZE]){
        self.prepare_for_transfer();
        
        self.dc_pin.set_low();
        self.write_raw(&[command]);
        self.dc_pin.set_high();
        self.write_raw(data);
        self.last_transfer_was_dma = false;
    }

    fn prepare_for_transfer(&mut self) {
        if self.last_transfer_was_dma{
            memory_barrier();   // Change SPI to DMA
            self.dma.end_dma_transfer();
            memory_barrier();   // Change DMA to SPI
            Self::setup_poll_fast_transfer(self.spi_registers);
        }
    }

    fn setup_poll_fast_transfer(spi_registers:&mut SpiRegisters){
        spi_registers.control_status.write(Self::SPI_CS_TA | Self::SPI_CS_CLEAR);

        // poll mode speed up according to this forum post - https://forums.raspberrypi.com/viewtopic.php?f=44&t=181154
        spi_registers.data_len.write(2);
    }

    fn write_raw<const SIZE:usize>(&mut self, data:&[u8;SIZE]){
        let mut current_index = 0;
        while current_index < SIZE {
            let cs:u32 = self.spi_registers.control_status.read();
            if cs & Self::SPI_CS_TXD != 0{
                self.spi_registers.fifo.write(data[current_index] as u32);
                current_index += 1;
            }
            if (cs & (Self::SPI_CS_RXR | Self::SPI_CS_RXF)) != 0 {
                self.spi_registers.control_status.write(Self::SPI_CS_TA | Self::SPI_CS_CLEAR_RX);
            }
        }

        // wait for the last trasfer to finish
        while (self.spi_registers.control_status.read() & Self::SPI_CS_DONE) == 0 {
            if (self.spi_registers.control_status.read() & (Self::SPI_CS_RXR | Self::SPI_CS_RXF)) != 0{
                self.spi_registers.control_status.write(Self::SPI_CS_TA | Self::SPI_CS_CLEAR_RX);
            }
        }
    }

    pub fn write_dma(&mut self, command:u8, data:&[u8;SPI_BUFFER_SIZE]){
        self.prepare_for_transfer();
        
        self.dc_pin.set_low();
        self.write_raw(&[command]);
        self.dc_pin.set_high();
        self.write_dma_raw(&data);
        self.last_transfer_was_dma = true;
    }


    // Since generic_const_exprs is not stable yet Im reserving the first 4 bytes of the data variable for internal use
    fn write_dma_raw(&mut self, data:&[u8;SPI_BUFFER_SIZE]){
        self.spi_registers.control_status.write(Self::SPI_CS_DMAEN | Self::SPI_CS_CLEAR);
        memory_barrier();   // Change SPI to DMA
        self.dma.start_dma_transfer(data, Self::SPI_CS_TA as u8);
        memory_barrier();   // Change DMA to SPI
    }

    pub fn fast_mode(&mut self) {
        self.spi_registers.clock.write(FAST_SPI_CLOCK_DIVISOR);
    }
}

impl Drop for Spi0{
    fn drop(&mut self) {
        self.spi_registers.control_status.write(Self::SPI_CS_CLEAR);
    }
}