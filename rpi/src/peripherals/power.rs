use super::{utils::{MmioReg32, get_static_peripheral, memory_barrier}, PERIPHERALS, Tag};

const RPI_DEVICES_COUNT:usize = if cfg!(rpi = "4") {11} else {9};

const PM_BASE_OFFSET:usize = 0x10_001C;

const PM_PASSWORD:u32 = 0x5A00_0000;
const PM_RSTC_WRCFG_CLR: u32 = 0xFFFF_FFCF;
const PM_RSTC_WRCFG_FULL_RESET: u32 = 0x0000_0020;
const PM_RSTS_PARTITION_MASK:u32 = 0xFFFF_FAAA;     // Bits 0,2,4,6,8,10 are cleared

#[repr(C)]
struct PowerRegisters{
    /// ## RSTC Register bit info:
    /// | Bit Index | Meaning |
    /// | --------- | ------- |
    /// | 0-3 | Unknown |
    /// | 4-5 | WRCFG value, 0b10 - Full Reset, 0b11 - Set | 
    /// |24-31| Password - 0x5A |
    rstc: MmioReg32,

    /// ## RSTS Register bits info:
    /// | Bit Index | Meaning |
    /// | --------- | ------- |
    /// | 0 | Bit 0 of boot partition |
    /// | 1 | Unknown |
    /// | 2 | Bit 1 of boot partition |
    /// | 3 | Unknown |
    /// | 4 | Bit 2 of boot partition |
    /// | 5 | Unknown |
    /// | 6 | Bit 3 of boot partition |
    /// | 7 | Unknown |
    /// | 8 | Bit 4 of boot partition |
    /// | 9 | Unknown |
    /// | 10 | Bit 5 of boot partition |
    /// |11-23| Unknown |
    /// |24-31| Password - 0x5A |
    rsts: MmioReg32,

    /// ## RSTS Register bits info:
    /// | Bit Index | Meaning |
    /// | --------- | ------- |
    /// |0-19| timeout ticks, 0 - disabled |
    /// |20-23| Unknown |
    /// |24-31| Password - 0x5A |
    wdog: MmioReg32
}


pub struct Power{
    registers: &'static mut PowerRegisters
}

// The RPI firmware uses the RSTS register to know to which partition to boot from
// The partition value is spread across non sequential bits (see the RSTS register description above)
// The enum is converted to u32 and the value is the value of those non sequential bits in the register 
#[repr(u32)]
pub enum ResetMode{
    Partition0 = 0,
    Halt = 0x555
}

impl Power{
    pub(super) fn new()->Self{
        Self { registers: get_static_peripheral(PM_BASE_OFFSET) }
    }

    pub fn reset(&mut self, mode:ResetMode){
        let mbox = unsafe{PERIPHERALS.get_mailbox()};
        for device_id in 0..RPI_DEVICES_COUNT{
            mbox.call(Tag::SetPowerState, [device_id as u32, 0 /* power off, no wait */]);
        }

        let gpio = unsafe{PERIPHERALS.get_gpio()};
        gpio.power_off();

        memory_barrier();
        let mut rsts_reg = self.registers.rsts.read();
        rsts_reg &= PM_RSTS_PARTITION_MASK;
        rsts_reg |= PM_PASSWORD | mode as u32;
        self.registers.rsts.write(rsts_reg);

        // Setting the watchdog timeout to a non zero value in order to trigger the reset
        self.registers.wdog.write(PM_PASSWORD | 1);

        let mut rstc_reg = self.registers.rstc.read();
        rstc_reg &= PM_RSTC_WRCFG_CLR;
        rstc_reg |= PM_PASSWORD | PM_RSTC_WRCFG_FULL_RESET;
        self.registers.rstc.write(rstc_reg);
        memory_barrier();
    }   
}