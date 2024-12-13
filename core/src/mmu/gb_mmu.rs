use super::{access_bus::AccessBus, carts::{Mbc, CGB_FLAG_ADDRESS}, external_memory_bus::{Bootrom, ExternalMemoryBus}, interrupts_handler::InterruptRequest, io_bus::IoBus, Memory};
use crate::{apu::{audio_device::AudioDevice, gb_apu::GbApu}, keypad::joypad_provider::JoypadProvider, machine::Mode, ppu::{color::Color, gfx_device::GfxDevice, ppu_state::PpuState}, utils::{bit_masks::{flip_bit_u8, BIT_7_MASK}, memory_registers::*}};

const HRAM_SIZE:usize = 0x7F;

const BAD_READ_VALUE:u8 = 0xFF;

pub struct GbMmu<'a, D:AudioDevice, G:GfxDevice, J:JoypadProvider>{
    io_bus: IoBus<D, G, J>,
    external_memory_bus:ExternalMemoryBus<'a>,
    occupied_access_bus:Option<AccessBus>,
    hram: [u8;HRAM_SIZE],
    double_speed_mode:bool,
    halt: bool,
    mode:Mode,
    #[cfg(feature = "dbg")]
    pub mem_watch: crate::debugger::MemoryWatcher,
}


//DMA only locks the used bus. there 2 possible used buses: extrnal (wram, rom, sram) and video (vram)
impl<'a, D:AudioDevice, G:GfxDevice, J:JoypadProvider> Memory for GbMmu<'a, D, G, J>{
    fn read(&mut self, address:u16, m_cycles:u8)->u8{
        #[cfg(feature = "dbg")]
        if self.mem_watch.watching_addresses.contains(&address){
            self.mem_watch.hit_addr = Some(address);
        }

        self.cycle(m_cycles);
        if let Some (bus) = &self.occupied_access_bus{
            return match address{
                0xFEA0..=0xFEFF | 0xFF00..=0xFFFF=>self.read_unprotected(address),
                0x8000..=0x9FFF => if let AccessBus::External = bus {self.read_unprotected(address)} else{Self::bad_dma_read(address)},
                0..=0x7FFF | 0xA000..=0xFDFF => if let AccessBus::Video = bus {self.read_unprotected(address)} else{Self::bad_dma_read(address)},
                _=>Self::bad_dma_read(address)
            };
        }
        return match address{
            0x8000..=0x9FFF=>{
                if self.is_vram_ready_for_io(){
                    return self.io_bus.ppu.vram.read_current_bank(address-0x8000);
                }
                else{
                    log::warn!("bad vram read");
                    return BAD_READ_VALUE;
                }
            },
            0xFE00..=0xFE9F=>{
                if self.is_oam_ready_for_io(){
                    return self.io_bus.ppu.oam[(address-0xFE00) as usize];
                }
                else{
                    log::warn!("bad oam read");
                    return BAD_READ_VALUE;
                }
            }
            _=>self.read_unprotected(address)
        };
    }

    fn write(&mut self, address:u16, value:u8, m_cycles:u8){
        #[cfg(feature = "dbg")]
        if self.mem_watch.watching_addresses.contains(&address){
            self.mem_watch.hit_addr = Some(address);
        }

        self.cycle(m_cycles);
        if let Some(bus) = &self.occupied_access_bus{
            match address{
                0xFF00..=0xFFFF=>self.write_unprotected(address, value),
                0x8000..=0x9FFF => if let AccessBus::External = bus {self.write_unprotected(address, value)} else{Self::bad_dma_write(address)},
                0..=0x7FFF | 0xA000..=0xFDFF => if let AccessBus::Video = bus {self.write_unprotected(address, value)} else{Self::bad_dma_write(address)},
                _=>Self::bad_dma_write(address)
            }
        }
        else{
            match address{
                0x8000..=0x9FFF=>{
                    if self.is_vram_ready_for_io(){
                        self.io_bus.ppu.vram.write_current_bank(address-0x8000, value);
                    }
                    else{
                        log::warn!("bad vram write: address - {:#X}, value - {:#X}, bank - {}", address, value, self.io_bus.ppu.vram.get_bank_reg());
                    }
                },
                0xFE00..=0xFE9F=>{
                    if self.is_oam_ready_for_io(){
                        self.io_bus.ppu.oam[(address-0xFE00) as usize] = value;
                    }
                    else{
                        log::warn!("bad oam write")
                    }
                },
                _=>self.write_unprotected(address, value)
            }
        }
    }

    fn set_double_speed_mode(&mut self, state:bool) {
        self.double_speed_mode = state;
    }

    fn set_halt(&mut self, state:bool) {
        self.halt = state;
    }
}

impl<'a, D:AudioDevice, G:GfxDevice, J:JoypadProvider> GbMmu<'a, D, G, J>{
    fn read_unprotected(&mut self, address:u16) ->u8 {
        return match address{
            0x0..=0x7FFF=>self.external_memory_bus.read(address),
            0x8000..=0x9FFF=>self.io_bus.ppu.vram.read_current_bank(address-0x8000),
            0xA000..=0xFDFF=>self.external_memory_bus.read(address),
            0xFE00..=0xFE9F=>self.io_bus.ppu.oam[(address-0xFE00) as usize],
            0xFEA0..=0xFEFF=>0x0,
            BOOT_REGISTER_ADDRESS => self.external_memory_bus.read_boot_reg(),
            SVBK_REGISTER_ADRRESS => self.external_memory_bus.read_svbk_reg(),
            0xFF00..=0xFF7F=>self.io_bus.read(address - 0xFF00),
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize],
            0xFFFF=>self.io_bus.interrupt_handler.interrupt_enable_flag
        };
    }

    fn write_unprotected(&mut self, address:u16, value:u8) {
        match address{
            0x0..=0x7FFF=>{
                self.external_memory_bus.write(address, value);
                #[cfg(feature = "dbg")]
                {
                    // Usually writes to this address range is used to swap rom bank
                    self.mem_watch.current_rom_bank_number = self.external_memory_bus.get_current_rom_bank();
                }
            },
            0x8000..=0x9FFF=>self.io_bus.ppu.vram.write_current_bank(address-0x8000, value),
            0xA000..=0xFDFF=>self.external_memory_bus.write(address, value),
            0xFE00..=0xFE9F=>self.io_bus.ppu.oam[(address-0xFE00) as usize] = value,
            0xFEA0..=0xFEFF=>{},
            BOOT_REGISTER_ADDRESS => {
                self.external_memory_bus.write_boot_reg(value);
                if self.external_memory_bus.finished_boot(){
                    self.io_bus.set_boot_finished();
                }
            },
            SVBK_REGISTER_ADRRESS => if self.io_bus.is_cgb_enabled() {
                self.external_memory_bus.write_svbk_reg(value);
                #[cfg(feature = "dbg")]
                {
                    self.mem_watch.current_ram_bank_number = self.external_memory_bus.get_current_ram_bank();
                }
            },
            0xFF00..=0xFF7F=>self.io_bus.write(address - 0xFF00, value),
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize] = value,
            0xFFFF=>self.io_bus.interrupt_handler.interrupt_enable_flag = value
        }
    }
}

impl<'a, D:AudioDevice, G:GfxDevice, J:JoypadProvider> GbMmu<'a, D, G, J>{
    pub fn new(mbc:&'a mut dyn Mbc, boot_rom:Option<Bootrom>, apu:GbApu<D>, gfx_device:G, joypad_proider:J, mode:Mode)->Self{
        let bootrom_missing = boot_rom.is_none();
        let cgb_reg = mbc.read_bank0(CGB_FLAG_ADDRESS as u16);
        let mut mmu = GbMmu{
            io_bus:IoBus::new(apu, gfx_device, joypad_proider, mode),
            external_memory_bus: ExternalMemoryBus::new(mbc, boot_rom),
            occupied_access_bus:None,
            hram:[0;HRAM_SIZE],
            double_speed_mode:false,
            halt: false,
            mode,
            #[cfg(feature = "dbg")]
            mem_watch: crate::debugger::MemoryWatcher::new()
        };
        if bootrom_missing{
            if mode == Mode::CGB {
                // Mimic the CGB bootrom behavior
                if cgb_reg & BIT_7_MASK != 0{
                    mmu.write(KEY0_REGISTER_ADDRESS, cgb_reg, 0);
                }
                else{
                    mmu.write(KEY0_REGISTER_ADDRESS, 0x4, 0);   // Set bit 2 that indicates DMG compatibility mode 
                    mmu.write(OPRI_REGISTER_ADDRESS, 1, 0);     // Set DMG priority mode
                    
                    // Default colors are from here - https://tcrf.net/Notes:Game_Boy_Color_Bootstrap_ROM
                    // Setup the default BG palettes
                    mmu.write(BGPI_REGISTER_ADDRESS, BIT_7_MASK, 0);    // Set to auto increment
                    mmu.write_color_ram(BGPD_REGISTER_ADDRESS, Color::from(0xFFFFFF as u32));
                    mmu.write_color_ram(BGPD_REGISTER_ADDRESS, Color::from(0x7BFF31 as u32));
                    mmu.write_color_ram(BGPD_REGISTER_ADDRESS, Color::from(0x0063C5 as u32));
                    mmu.write_color_ram(BGPD_REGISTER_ADDRESS, Color::from(0x000000 as u32));

                    // Setup the default OBJ palettes
                    mmu.write(OBPI_REGISTER_ADDRESS, BIT_7_MASK, 0);    // Set to auto increment
                    // Fill the first 2 object palettes with the same palette
                    for _ in 0..2{
                        mmu.write_color_ram(OBPD_REGISTER_ADDRESS, Color::from(0xFFFFFF as u32));
                        mmu.write_color_ram(OBPD_REGISTER_ADDRESS, Color::from(0xFF8484 as u32));
                        mmu.write_color_ram(OBPD_REGISTER_ADDRESS, Color::from(0x943A3A as u32));
                        mmu.write_color_ram(OBPD_REGISTER_ADDRESS, Color::from(0x000000 as u32));
                    }
                }
            }
            //Setting the bootrom register to be set (the boot sequence has over)
            mmu.write(BOOT_REGISTER_ADDRESS, 1, 0);
        }

        return mmu;
    }

    pub fn cycle(&mut self, m_cycles:u8){
        flip_bit_u8(&mut self.io_bus.speed_switch_register, 7, self.double_speed_mode);
        self.occupied_access_bus = self.io_bus.cycle(m_cycles as u32, self.double_speed_mode, self.halt, &mut self.external_memory_bus);
    }

    pub fn handle_interrupts(&mut self, master_interrupt_enable:bool)->InterruptRequest{
        return self.io_bus.interrupt_handler.handle_interrupts(master_interrupt_enable, self.io_bus.ppu.stat_register);
    }

    pub fn poll_joypad_state(&mut self){
        self.io_bus.joypad_handler.poll_joypad_state();
    }

    pub fn dma_block_cpu(&self)->bool{
        return match self.mode {
            Mode::DMG => false,
            Mode::CGB => self.io_bus.vram_dma_controller.should_block_cpu(),
        };
    }

    pub fn consume_vblank_event(&mut self)->bool{self.io_bus.ppu.consume_vblank_event()}

    #[cfg(feature = "dbg")]
    pub fn get_ppu(&self)->&crate::ppu::gb_ppu::GbPpu<G>{&self.io_bus.ppu}

    #[cfg(feature = "dbg")]
    pub fn dbg_read(&mut self, address:u16)->u8{self.read_unprotected(address)}

    fn is_oam_ready_for_io(&self)->bool{
        return self.io_bus.ppu.state != PpuState::OamSearch && self.io_bus.ppu.state != PpuState::PixelTransfer
    }

    fn is_vram_ready_for_io(&self)->bool{
        return self.io_bus.ppu.state != PpuState::PixelTransfer;
    }

    fn bad_dma_read(address:u16)->u8{
        log::warn!("bad memory read during dma. {:#X}", address);
        return BAD_READ_VALUE;
    }

    fn bad_dma_write(address:u16){
        log::warn!("bad memory write during dma. {:#X}", address)
    }

    fn write_color_ram(&mut self, address:u16, color: Color){
        let bgr555_value = ((color.r >> 3) as u16 & 0b1_1111) | (((color.g >> 3) as u16 & 0b1_1111) << 5)  | (((color.b >> 3) as u16 & 0b1_1111) << 10);
        
        // The value is little endian in memory so writing the low bits first and then the high bits
        self.write(address, (bgr555_value & 0xFF) as u8, 0);
        self.write(address, ((bgr555_value >> 8) & 0xFF) as u8, 0);
    }
}