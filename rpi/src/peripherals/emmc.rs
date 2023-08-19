// This module was wrriten with the great help of the LLD tutorial (https://github.com/rockytriton/LLD)
// and the Circle baremetal framework/library

use core::mem::size_of;

use bitfield_struct::bitfield;

use crate::{delay, peripherals::{InputGpioPin, IoGpioPin}};
use super::{utils::{MmioReg32, compile_time_size_assert, self, memory_barrier}, PERIPHERALS, Mailbox};

// 0x30_0000 - EMMC1 for other RPI's, 0x34_0000 - EMMC2 for RPI4
const EMMC_BASE_OFFSET:usize = if cfg!(feature = "rpi4"){0x34_0000} else {0x30_0000};

const SD_INIT_CLOCK:u32     = 400000;
const SD_NORMAL_CLOCK:u32   = 25000000;

const BLOCK_SIZE:u32 = 512;

#[derive(Debug)]
enum SdError{
    Timeout,
    CommandTimeout,
    Error,
}

#[repr(C, align(4))]
struct EmmcRegisters{
    arg2:MmioReg32,
    blksizecnt:MmioReg32,
    arg1:MmioReg32,
    cmdtm:MmioReg32,
    resp:[MmioReg32;4],
    data:MmioReg32,
    status:MmioReg32,
    control:[MmioReg32;2],
    interrupt:MmioReg32,
    irpt_mask:MmioReg32,
    irpt_en:MmioReg32,
    control2:MmioReg32,
    _pad:[u32;0x2F],
    slotisr_ver:MmioReg32
}
compile_time_size_assert!(EmmcRegisters, 0x100);

#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
enum CommandResponseType{
    _None = 0, 
    B136 = 1,
    B48 = 2,
    B48Busy = 3
}

impl CommandResponseType{
    const fn into_bits(self)->u32{self as _}
    const fn from_bits(value:u32)->Self{unsafe{core::mem::transmute(value & 0b11)}}
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
enum SdCommandType{
    GoIdle = 0,
    SendCide = 2,
    SendRelativeAddr = 3,
    IoSetOpCond = 5,
    SelectCard = 7,
    SendIfCond = 8,
    SetBlockLen = 16,
    ReadBlock = 17,
    ReadMultiple = 18,
    WriteBlock = 24,
    WriteMultiple = 25,
    OcrCheck = 41,
    SendScr = 51,
    App = 55
}

#[bitfield(u32)]
struct SdCommand{
    resp_a: bool,
    block_count: bool,
    #[bits(2)]auto_command: u8,
    direction: bool,
    multiblock: bool,
    #[bits(10)]resp_b: u16,
    #[bits(2)]response_type: CommandResponseType,
    __ : bool,
    crc_enable: bool,
    idx_enable: bool,
    is_data: bool,
    #[bits(2)]kind: u8,
    #[bits(6)]index: u8,
    __ : bool,

    // This field is my invention (took inspiration from Circle) in order to mark app commands, 
    // it uses the last bit who is reserved (it has no other purpose)
    // it is later cleared by the get_command function
    app_command:bool,
}

impl SdCommand{
    fn get_command(self)->u32{
        self.with_app_command(false).0
    }
}

const fn resolve_command(command_type:SdCommandType)->SdCommand{
    let command = SdCommand::new().with_index(command_type as u8);
    return match command_type{
        SdCommandType::GoIdle           => command,
        SdCommandType::SendCide         => command.with_response_type(CommandResponseType::B136).with_crc_enable(true),
        SdCommandType::SendRelativeAddr => command.with_response_type(CommandResponseType::B48).with_crc_enable(true),
        SdCommandType::IoSetOpCond      => command.with_response_type(CommandResponseType::B136),
        SdCommandType::SelectCard       => command.with_response_type(CommandResponseType::B48Busy).with_crc_enable(true),
        SdCommandType::SendIfCond       => command.with_response_type(CommandResponseType::B48).with_crc_enable(true),
        SdCommandType::SetBlockLen      => command.with_response_type(CommandResponseType::B48).with_crc_enable(true),
        SdCommandType::ReadBlock        => command.with_direction(true).with_response_type(CommandResponseType::B48).with_crc_enable(true).with_is_data(true),
        SdCommandType::ReadMultiple     => command.with_block_count(true).with_auto_command(1).with_direction(true).with_multiblock(true)
                                            .with_response_type(CommandResponseType::B48).with_crc_enable(true).with_is_data(true),
        SdCommandType::OcrCheck         => command.with_response_type(CommandResponseType::B48).with_app_command(true),
        SdCommandType::SendScr          => command.with_direction(true).with_response_type(CommandResponseType::B48).with_crc_enable(true).with_is_data(true).with_app_command(true),
        SdCommandType::App              => command.with_response_type(CommandResponseType::B48).with_crc_enable(true),
        _=> unreachable!()
    };
}

const CONTROL1_SRST_CMD:u32     = 1 << 25;
const CONTROL1_SRST_HC:u32      = 1 << 24;
const CONTROL1_DATA_TOUNIT:u32  = 0xF << 16;
const CONTROL1_CLK_FREQ8:u32    = 0xFF << 8;
const CONTORL1_CLK_FREQ_MS2:u32 = 0b11 << 6;
const CONTROL1_CLK_FREQ:u32     = CONTORL1_CLK_FREQ_MS2 | CONTROL1_CLK_FREQ8;
const CONTROL1_CLK_EN:u32        = 1 << 2;
const CONTROL1_CLK_STABLE:u32   = 1 << 1;
const CONTROL1_CLK_INTLEN:u32   = 1;
const STATUS_DAT_INHINIT:u32    = 1 << 1;
const STATUS_CMD_INHIBIT:u32    = 1;
const INTERRUPT_ACMD_ERR:u32    = 1 << 24;
const INTERRUPT_DEND_ERR:u32    = 1 << 22;
const INTERRUPT_DCRC_ERR:u32    = 1 << 21;
const INTERRUPT_DTO_ERR:u32     = 1 << 20;
const INTERRUPT_CBAD_ERR:u32    = 1 << 19;
const INTERRUPT_CEND_ERR:u32    = 1 << 18;
const INTERRUPT_CCRC_ERR:u32    = 1 << 17;
const INTERRUPT_CTO_ERR:u32     = 1 << 16;
const INTERRUPT_ERR:u32         = 1 << 15;
const INTERRUPT_READ_RDY:u32    = 1 << 5;
const INTERRUPT_WRITE_RDY:u32   = 1 << 4;
const INTERRUPT_DATA_DONE:u32   = 1 << 1;
const INTERRUPT_CMD_DONE:u32    = 1;
const INTERRUPT_ERROR_MASK:u32  = INTERRUPT_CTO_ERR | INTERRUPT_CCRC_ERR | INTERRUPT_CEND_ERR | INTERRUPT_CBAD_ERR | 
    INTERRUPT_DTO_ERR | INTERRUPT_DCRC_ERR | INTERRUPT_DEND_ERR | INTERRUPT_ACMD_ERR;

const CONTROL1_DATA_TOUNIT_MAX_VAL:u32  = 0xE << 16;

struct Scr{
    register:[u32;2],
    version:u32,
}

// Mailbox Tags
const SET_GPIO_STATE_TAG:u32        = 0x38041;
const GPIO_TAG_PIN_1_8V_CONTROL:u32 = 132;
const SET_POWER_STATE_TAG:u32       = 0x28001;
const SD_CARD_DEVICE_ID:u32         = 0;
const POWER_SET_TAG_ON:u32          = 1;
const POWER_SET_TAG_WAIT:u32        = 1 << 1;
const POWER_SET_TAG_NO_DEVICE:u32   = 1 << 1;

const GET_CLOCK_RATE_TAG:u32        = 0x30002;
// In RPI4 take the EMMC2 clock
const EMMC_CLOCK_ID:u32             = if cfg!(feature = "rpi4") {0xC} else {0x1};

pub struct Emmc{
    registers: &'static mut EmmcRegisters,
    _hw_version:u32,
    last_response:[u32;4],
    ocr:u32,                                // Short for Operation Condition Register (https://luckyresistor.me/cat-protector/software/sdcard-2/)
    sdhc_support:bool,
    rca:u32,                                // Short for Relative Card Address
    scr:Scr,                                // Short for SD Configuration Register - (http://problemkaputt.de/gbatek-dsi-sd-mmc-protocol-scr-register-64bit-sd-card-configuration-register.htm)
    block_size:u32,
    transfer_blocks:u32,

    offset:u64,                             // pointer to the current address being operated on
    // only for raspi3
    _ocupied_input_pins:[InputGpioPin;6],
    _ocupied_io_pins:[IoGpioPin;6],
}

impl Emmc{
    pub(super) fn new()->Self{
        let registers:&mut EmmcRegisters = utils::get_static_peripheral(EMMC_BASE_OFFSET);

        // only relevant for RPI3 and lower but seems unharmful
        let gpio = unsafe{PERIPHERALS.get_gpio()};
        let input_pins = [
            gpio.take_pin(34).into_input(super::GpioPull::None),
            gpio.take_pin(35).into_input(super::GpioPull::None),
            gpio.take_pin(36).into_input(super::GpioPull::None),
            gpio.take_pin(37).into_input(super::GpioPull::None),
            gpio.take_pin(38).into_input(super::GpioPull::None),
            gpio.take_pin(39).into_input(super::GpioPull::None),
        ];
        let io_pins = [
            gpio.take_pin(48).into_io(super::Mode::Alt3),
            gpio.take_pin(49).into_io(super::Mode::Alt3),
            gpio.take_pin(50).into_io(super::Mode::Alt3),
            gpio.take_pin(51).into_io(super::Mode::Alt3),
            gpio.take_pin(52).into_io(super::Mode::Alt3),
            gpio.take_pin(53).into_io(super::Mode::Alt3),
        ];

        // read hardware version
        let hw_version = (registers.slotisr_ver.read() & 0x00FF_0000) >> 16;     
        log::debug!("EMMC HW version: {}", hw_version);

        return Self { 
            registers, _hw_version: hw_version, last_response:[0;4], ocr:0, sdhc_support:false, rca:0, scr:Scr{register:[0;2], version:0},
            block_size: BLOCK_SIZE, transfer_blocks: 0, offset:0, _ocupied_input_pins: input_pins, _ocupied_io_pins: io_pins
        };
    }

    pub fn init(&mut self){
        memory_barrier();

        let mbox = unsafe{PERIPHERALS.get_mailbox()};
        // this code is only for the RPI4
        #[cfg(feature = "rpi4")]
        {
            let res = mbox.call(SET_GPIO_STATE_TAG, [GPIO_TAG_PIN_1_8V_CONTROL/* Pin to operate on */, 0/* requested state */]);
            // Test for the GPIO pin state set
            if res[1] != 0{
                core::panic!("Failed to disable RPI4 GPIO 1.8v supply");
            }
        }

        self.power_on(mbox);

        // reset the card
        self.registers.control[0].write(0);
        let control1 = self.registers.control[1].read();
        self.registers.control[1].write(control1 | CONTROL1_SRST_HC);
        Self::wait_timeout(&self.registers.control[1], CONTROL1_SRST_HC, true, 10, 100)
            .expect("Could not reset the SD card");

        // this block is only for RPI4
        #[cfg(feature = "rpi4")]
        {
            let mut control0 = self.registers.control[0].read();
            control0 |= 0xF00;      // Those bits are not documented in bcm2835 docs, it is copied from Circle
            self.registers.control[0].write(control0);
            delay::wait_ms(2);
        }   

        self.setup_peripheral_clock(mbox);

        self.registers.irpt_en.write(0xFFFF_FFFF);
        self.registers.irpt_mask.write(0xFFFF_FFFF);

        self.send_command(SdCommandType::GoIdle, 0, 2000, None).unwrap();
        
        let v2card = self.check_v2card();
        self.verify_usable_card();
        self.verify_ocr();
        self.verify_sdhc_support(v2card);

        self.switch_clock_rate(SD_NORMAL_CLOCK, mbox);

        delay::wait_ms(10);

        self.verify_rca();
        self.select_card();
        self.set_scr();

        self.registers.interrupt.write(0xFFFF_FFFF);

        memory_barrier();

        log::info!("Initialized the EMMC controller");
    }

    pub fn seek(&mut self, offset:u64){
        self.offset = offset;
    }

    pub fn read(&mut self, buffer:&mut [u8])->bool{
        if self.offset % BLOCK_SIZE as u64 != 0{
            return false;
        }
        
        let block = self.offset / BLOCK_SIZE as u64;

        memory_barrier();
        let result = self.execute_data_transfer_command(false, buffer, block as u32).is_ok();
        memory_barrier();
        return result;
    }

    pub fn get_block_size(&self)->u32{self.block_size}

    fn execute_data_transfer_command(&mut self, write: bool, buffer: &mut [u8], mut block_index:u32)->Result<(), SdError>{
        if !self.sdhc_support{
            block_index *= BLOCK_SIZE;
        }
        let buffer_size = buffer.len() as u32;
        if buffer_size < self.block_size {
            log::warn!("buffer is smaller than block size. buffer: {}, block size: {}", buffer_size, self.block_size);
            return Err(SdError::Error);
        }
        if buffer_size % self.block_size != 0{
            log::warn!("buffer does not fit into a block size. buffer: {}, block_size: {}", buffer_size, self.block_size);
            return Err(SdError::Error);
        }

        self.transfer_blocks = buffer_size / self.block_size;

        let command_type = if write && self.transfer_blocks > 1{SdCommandType::WriteMultiple}
            else if !write && self.transfer_blocks > 1 {SdCommandType::ReadMultiple} 
            else if write{SdCommandType::WriteBlock}
            else{SdCommandType::ReadBlock};
        
        for _ in 0..3{
            if self.send_command(command_type, block_index, 5000, Some(buffer)).is_ok(){
                return Ok(());
            }
        }
        log::warn!("Failed data command with {} blocks and size of: {}", self.transfer_blocks, buffer_size);

        return Err(SdError::Error);
    }
    
    fn check_v2card(&mut self)->bool{
        if self.send_command(SdCommandType::SendIfCond, 0x1AA, 200, None).is_err(){
            return false;
        }
        if self.last_response[0] & 0xFFF != 0x1AA{
            return false;
        }   
        return true;
    }

    fn verify_usable_card(&mut self){
        if let Err(error_type) = self.send_command(SdCommandType::IoSetOpCond, 0, 1000, None){
            if let SdError::CommandTimeout = error_type{
                if self.reset_device_command_handling(){
                    return;
                }
            }
            core::panic!("Sd card failed usable card verify")
        }
    }

    fn verify_ocr(&mut self){
        self.send_command(SdCommandType::OcrCheck, 0, 2000, None).unwrap();
        self.ocr = (self.last_response[0] >> 8) & 0xFFFF;
    }

    fn verify_sdhc_support(&mut self, v2card:bool){
        let mut card_busy = true;
        while card_busy {
            let v2flags = if v2card {1<<30} else {0};
            self.send_command(SdCommandType::OcrCheck, 0x00FF_8000 | v2flags, 2000, None).unwrap();
            let response = self.last_response[0];
            if response >> 31 & 1 != 0{
                self.ocr = (response >> 8) & 0xFFFF;
                self.sdhc_support = (response >> 30) & 1 != 0;
                card_busy = false;
            }
            else{
                delay::wait_ms(500);
            }
        }
    }

    fn verify_rca(&mut self){
        self.send_command(SdCommandType::SendCide, 0, 2000, None).unwrap();
        self.send_command(SdCommandType::SendRelativeAddr, 0, 2000, None).unwrap();
        self.rca = (self.last_response[0] >> 16) & 0xFFFF;
        if (self.last_response[0] >> 8) & 1 == 0{
            core::panic!("Failed to read RCA");
        }
    }

    fn select_card(&mut self){
        self.send_command(SdCommandType::SelectCard, self.rca << 16, 2000, None).unwrap();
        let status = (self.last_response[0] >> 9) & 0xF;
        if status != 3 && status != 4{
            core::panic!("Invalid Select Card status");
        }
    }

    fn set_scr(&mut self){
        if !self.sdhc_support{
            self.send_command(SdCommandType::SetBlockLen, BLOCK_SIZE, 2000, None).unwrap();
        }
        let mut block_size_count = self.registers.blksizecnt.read();
        block_size_count &= !0xFFF;                     // mask out the lower bits, only the first 10 bits matters the rest are reserved until 16
        block_size_count |= BLOCK_SIZE;                 // set the block size
        self.registers.blksizecnt.write(block_size_count);

        let mut scr_buffer = [0;8];
        self.block_size = 8;
        self.transfer_blocks = 1;
        self.send_command(SdCommandType::SendScr, 0, 30000, Some(&mut scr_buffer)).unwrap();
        // continue later
        self.block_size = BLOCK_SIZE;
        self.scr.register[0] = u32::from_ne_bytes(scr_buffer[0..4].try_into().unwrap());
        self.scr.register[1] = u32::from_ne_bytes(scr_buffer[4..].try_into().unwrap());

        // Note that SCR register is big endian
        let scr0 = self.scr.register[0].swap_bytes();
        let spec = (scr0 >> (56 - 32)) & 0xF;
        // spec3 and spec4 are supposed to be 1 bit according to this - 
        // http://problemkaputt.de/gbatek-dsi-sd-mmc-protocol-scr-register-64bit-sd-card-configuration-register.htm
        let spec3 = (scr0 >> (47 - 32)) & 0x1;
        let spec4 = (scr0 >> (42 - 32)) & 0x1;

        self.scr.version = match spec{
            0 => 1,
            1 => 11,
            2 if spec3 == 0 => 2,
            2 if spec3 == 1 =>{
                if spec4 == 0 {3} 
                else if spec4 == 1 {4}
                else {0}
            },
            _=> 0
        };
        if self.scr.version == 0{
            log::warn!("Couldnt verify SD spec version, SCR: {:#X}", scr0);
        }
        log::debug!("SD Spec version: {}", self.scr.version);
    }

    fn transfer_data(&mut self, command:SdCommand, buffer:&mut [u8]){
        let (wr_irpt, write) = if command.direction(){(INTERRUPT_READ_RDY, false)} else{(INTERRUPT_WRITE_RDY, true)};

        for i in 0..self.transfer_blocks{
            if let Err(_) = Self::wait_timeout(&self.registers.interrupt, wr_irpt | INTERRUPT_ERR, false, 1, 2000){
                let intr_val = self.registers.interrupt.read();
                if intr_val & (INTERRUPT_ERROR_MASK | wr_irpt) != wr_irpt{
                    core::panic!("Error while transfering, interrupt {:#X}, iteration: {}", intr_val, i);
                }
            }
            self.registers.interrupt.write(wr_irpt | INTERRUPT_ERR);

            const DATA_REG_SIZE:usize = size_of::<MmioReg32>();
            let iteration_len = self.block_size as usize / DATA_REG_SIZE;
            let block_index = i as usize * self.block_size as usize;   
            if write{
                for j in 0..iteration_len{
                    let data:[u8; DATA_REG_SIZE as usize] = buffer[block_index + (j * DATA_REG_SIZE) .. block_index + ((j + 1) * DATA_REG_SIZE)].try_into().unwrap();
                    self.registers.data.write(u32::from_ne_bytes(data));
                }
            }
            else{
                for j in 0..iteration_len{
                    let data = u32::to_ne_bytes(self.registers.data.read());
                    buffer[block_index + (j * DATA_REG_SIZE) .. block_index + ((j + 1) * DATA_REG_SIZE)].copy_from_slice(&data);
                }
            }
        }
    }

    fn reset_device_command_handling(&mut self)->bool{
        let control1 = self.registers.control[1].read();
        self.registers.control[1].write(control1 | CONTROL1_SRST_CMD);
        return Self::wait_timeout(&self.registers.control[1], CONTROL1_SRST_CMD, true, 1, 10000).is_ok();
    }

    fn send_command(&mut self, command_type:SdCommandType, arg:u32, timeout_ms:u32, buffer:Option<&mut [u8]>)->Result<(), SdError>{
        log::trace!("Received command type: {:#?}", command_type);
        let command = resolve_command(command_type);

        if command.app_command(){
            log::trace!("Sending App command");
            let app_command = resolve_command(SdCommandType::App);
            let rca = self.rca << 16;   
            self.send_raw_command(app_command, rca, 2000, None)?;
        }

        return self.send_raw_command(command, arg, timeout_ms, buffer);
    }
    
    fn send_raw_command(&mut self, command:SdCommand, arg:u32, timeout_ms:u32, buffer:Option<&mut [u8]>)->Result<(), SdError>{
        log::trace!("Command {:#X} is being processed", command.0);

        let block_size_count_value = self.block_size | (self.transfer_blocks << 16);
        self.registers.blksizecnt.write(block_size_count_value);
        self.registers.arg1.write(arg);
        self.registers.cmdtm.write(command.get_command());

        delay::wait_ms(10);

        Self::wait_timeout(&self.registers.interrupt, INTERRUPT_CMD_DONE | INTERRUPT_ERR, false, 1, timeout_ms)?;

        let intr_val = self.registers.interrupt.read();
        self.registers.interrupt.write(INTERRUPT_ERROR_MASK | INTERRUPT_CMD_DONE);

        if intr_val & INTERRUPT_ERROR_MASK | INTERRUPT_CMD_DONE != 1{
            return if intr_val & INTERRUPT_CTO_ERR != 0 {Err(SdError::CommandTimeout)} else {Err(SdError::Timeout)};
        }

        match command.response_type(){
            CommandResponseType::B48 |
            CommandResponseType::B48Busy => self.last_response[0] = self.registers.resp[0].read(),
            CommandResponseType::B136 => {
                self.last_response[0] = self.registers.resp[0].read();
                self.last_response[1] = self.registers.resp[1].read();
                self.last_response[2] = self.registers.resp[2].read();
                self.last_response[3] = self.registers.resp[3].read();
            },
            CommandResponseType::_None => {},
        }

        if command.is_data(){
            self.transfer_data(command, buffer.unwrap());
        }
        
        if command.response_type() == CommandResponseType::B48Busy || command.is_data(){
            Self::wait_timeout(&self.registers.interrupt, INTERRUPT_ERROR_MASK | INTERRUPT_DATA_DONE, false, 1, 2000)?;
            let intr_val = self.registers.interrupt.read();
            if intr_val & INTERRUPT_ERROR_MASK | INTERRUPT_DATA_DONE != INTERRUPT_DATA_DONE && 
                intr_val & INTERRUPT_ERROR_MASK | INTERRUPT_DATA_DONE != INTERRUPT_DTO_ERR | INTERRUPT_DATA_DONE{
                return Err(SdError::Error);
            }

            self.registers.interrupt.write(INTERRUPT_ERROR_MASK | INTERRUPT_DATA_DONE);
        }

        return Ok(());
    }

    fn wait_timeout(register:&MmioReg32, mask:u32, wait_clear:bool, retry_ms:u32, timeout_retries:u32)->Result<(), SdError>{
        for _ in 0..timeout_retries{
            let value = register.read() & mask == 0;
            if value == wait_clear{
                return Result::Ok(());
            }
            delay::wait_ms(retry_ms);
        }
        return Result::Err(SdError::Timeout);
    }

    fn power_on(&mut self, mbox: &mut Mailbox){
        let res = mbox.call(SET_POWER_STATE_TAG, [SD_CARD_DEVICE_ID, POWER_SET_TAG_ON | POWER_SET_TAG_WAIT]);
        if res[1] & POWER_SET_TAG_ON == 0 || res[1] & POWER_SET_TAG_NO_DEVICE != 0{
            core::panic!("Could not power on the SD card device from the mbox interface");
        }
    }

    fn switch_clock_rate(&mut self, target_rate:u32, mbox: &mut Mailbox){
        let base_clock = self.get_base_clock(mbox);
        Self::wait_timeout(&self.registers.status, STATUS_CMD_INHIBIT | STATUS_DAT_INHINIT, true, 1, 1000)
            .expect("Error!, data or command lines are still busy");

        // turn off the clock
        let control1 = self.registers.control[1].read() & !CONTROL1_CLK_EN;
        self.registers.control[1].write(control1);
        delay::wait_ms(3);

        // Set the clock freq div
        let divider = Self::get_clock_divider_register(base_clock, target_rate);
        self.registers.control[1].write((control1 & !CONTROL1_CLK_FREQ) | divider );
        delay::wait_ms(3);

        let control1 = self.registers.control[1].read();
        self.registers.control[1].write(control1 | CONTROL1_CLK_EN);
        self::delay::wait_ms(3);
    }


    fn setup_peripheral_clock(&mut self, mbox:&mut Mailbox){
        self.registers.control2.write(0);   // clear according to Circle and LLD
        let clock_rate = self.get_base_clock(mbox);

        let mut control1 = self.registers.control[1].read();
        control1 |= CONTROL1_CLK_INTLEN;
        control1 |= Self::get_clock_divider_register(clock_rate, SD_INIT_CLOCK);
        control1 &= !CONTROL1_DATA_TOUNIT;
        control1 |= CONTROL1_DATA_TOUNIT_MAX_VAL;
        self.registers.control[1].write(control1);

        Self::wait_timeout(&self.registers.control[1], CONTROL1_CLK_STABLE, false, 10, 100)
            .expect("Could not achive a stable clock");

        control1 = self.registers.control[1].read();
        self.registers.control[1].write(control1 | CONTROL1_CLK_EN)
    }

    fn get_base_clock(&mut self, mbox:&mut Mailbox)->u32{
        let res = mbox.call(GET_CLOCK_RATE_TAG, [EMMC_CLOCK_ID, 0]);
        let clock_rate = res[1];
        return clock_rate;
    }

    fn get_clock_divider_register(base_clock:u32, target_rate:u32)->u32{
        let mut target_div = 1;
        if target_rate <= base_clock{
            target_div = base_clock / target_rate;
            if base_clock % target_rate != 0{
                target_div = 0;
            }
        }
        let mut div:i32 = -1;
        for fb in (0..=31).rev(){
            let bit = 1 << fb;
            if target_div & bit != 0{
                div = fb;
                target_div &= !bit;
                if target_div != 0{
                    div += 1;
                }
                break;
            }
        }
        if div == -1{
            div = 31;
        }
        else if div >= 32{
            div = 31;
        }
        if div != 0{
            div = 1 << (div - 1);
        }
        if div >= 0x400{
            div = 0x3FF;
        }

        // the register requires first the 2 MSB bits and than the 8 LSB bits
        let lower_bits = div & 0xFF;
        let upper_bits = (div >> 8) & 0b11;
        return ((lower_bits << 8) | upper_bits << 6) as u32;
    }
}