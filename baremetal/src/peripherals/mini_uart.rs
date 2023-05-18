use super::{Mode, GpioPull, CORE_FREQ, PERIPHERALS_BASE_ADDRESS, utils::{MmioReg32, compile_time_size_assert, memory_barrier}, PERIPHERALS};

const UART_RX_PIN_BCM: u8 = 15;
const UART_TX_PIN_BCM: u8 = 14;
const AUX_BASE_ADDRESS:usize = PERIPHERALS_BASE_ADDRESS + 0x21_5000;
const AUX_MINI_UART_ADDRESS:usize = AUX_BASE_ADDRESS + 0x40;

#[repr(C, align(4))]
struct AuxControlRegisters{
    aux_irq:MmioReg32,
    aux_enables:MmioReg32,
}

#[repr(C, align(4))]
struct MiniUartRegisters{
    io:MmioReg32,         // data
    ier:MmioReg32,        // interrupt enable
    iir:MmioReg32,        // imterrupt identity 
    lcr:MmioReg32,        // line control
    mcr:MmioReg32,        // modem control
    lsr:MmioReg32,        // line status
    _msr:MmioReg32,        // modem status
    _scratch:MmioReg32,
    cntl:MmioReg32,       // extra control
    _stat:MmioReg32,       // extra status
    baud:MmioReg32
}
compile_time_size_assert!(MiniUartRegisters, 0x2C);

/* The docs are wrong and this is a 3 bit parameter: 
According the Linux kernel header - https://git.kernel.org/pub/scm/linux/kernel/git/next/linux-next.git/tree/include/uapi/linux/serial_reg.h#n110
5bit - 0
6bit - 1
7bit - 2
8bit - 3 */
const AUX_MU_LCR_8BIT_DATA_SIZE:u32 = 0b11;
const AUX_MU_IIR_CLEAR_FIFO:u32     = 0b11 << 1;
const AUX_MU_CNTL_ENABLE_RX:u32     = 1;
const AUX_MU_CNTL_ENABLE_TX:u32     = 1 << 1;
const AUX_MU_LSR_TX_EMPTY:u32       = 1 << 5;
const AUX_ENABLES_ENABLE_UART:u32   = 1;

pub struct MiniUart{
    registers:&'static mut MiniUartRegisters,
    _aux_control_registers:&'static AuxControlRegisters
}

impl MiniUart{
    pub(super) fn new(baudrate:u32)->MiniUart{
        let gpio = unsafe{PERIPHERALS.get_gpio()};
        // alt5 is the uart1 which the mini uart uses
        let mut rx_pin = gpio.take_pin(UART_RX_PIN_BCM, Mode::Alt5);
        let mut tx_pin = gpio.take_pin(UART_TX_PIN_BCM, Mode::Alt5);

        // set pull to none for the pins
        rx_pin.set_pull(GpioPull::None);
        tx_pin.set_pull(GpioPull::None);

        // the docs for bcm2835 says I might need to sleep here for 150 cycles, the bcm2711 omitted this and changed the registers

        let control_regs = unsafe{&mut *(AUX_BASE_ADDRESS as *mut AuxControlRegisters)};
        let mini_uart_regs = unsafe{&mut *(AUX_MINI_UART_ADDRESS as *mut MiniUartRegisters)};

        // setup uart
        memory_barrier();
        control_regs.aux_enables.write(AUX_ENABLES_ENABLE_UART);    // enables uart
        mini_uart_regs.ier.write(0);                            // turn off interrupts
        mini_uart_regs.cntl.write(0);                           // turn off io for init
        mini_uart_regs.lcr.write(AUX_MU_LCR_8BIT_DATA_SIZE);
        mini_uart_regs.mcr.write(0);                            // set rts to high
        mini_uart_regs.iir.write(AUX_MU_IIR_CLEAR_FIFO);                            
        mini_uart_regs.baud.write(Self::aux_mu_baud(baudrate));
        mini_uart_regs.cntl.write(AUX_MU_CNTL_ENABLE_RX | AUX_MU_CNTL_ENABLE_TX);
        memory_barrier();

        // the pins are leaking right now, but I dont care
        return MiniUart{registers:mini_uart_regs, _aux_control_registers:control_regs};
    }

    pub fn send(&mut self, data:&[u8]){
        memory_barrier();
        for ch in data{
            // block untill we can write
            while (self.registers.lsr.read() & AUX_MU_LSR_TX_EMPTY) == 0 {}
            // TX is empty writing to the fifo register
            self.registers.io.write(*ch as u32);
        }
        memory_barrier();
    }

    // baudrate = core_freq / (8(baudrate_reg + 1))
    const fn aux_mu_baud(baudrate:u32)->u32{(CORE_FREQ / (8*baudrate)) - 1}
}