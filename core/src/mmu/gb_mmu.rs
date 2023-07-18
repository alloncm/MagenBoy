use super::{
    Memory, carts::Mbc, external_memory_bus::{ExternalMemoryBus, Bootrom}, 
    interrupts_handler::InterruptRequest, io_bus::IoBus, access_bus::AccessBus
};
use crate::{
    ppu::{ppu_state::PpuState, gfx_device::GfxDevice}, keypad::joypad_provider::JoypadProvider, 
    utils::{bit_masks::flip_bit_u8, memory_registers::BOOT_REGISTER_ADDRESS, CYCLES_PER_FRAME}, apu::{audio_device::AudioDevice, gb_apu::GbApu}, machine::Mode
};

const HRAM_SIZE:usize = 0x7F;

const BAD_READ_VALUE:u8 = 0xFF;

cfg_if::cfg_if!{if #[cfg(feature = "dbg")]{
    pub struct MemoryWatcher{
        watching_addrs:crate::utils::FixedSizeSet<u16, 0xFF>,
        pub hit_addr:Option<u16>,
    }
    
    impl MemoryWatcher{
        pub fn add_address(&mut self, address:u16){self.watching_addrs.add(address)}
        pub fn try_remove_address(&mut self, address:u16)->bool{self.watching_addrs.try_remove(address)}
    }
}}

pub struct GbMmu<'a, D:AudioDevice, G:GfxDevice, J:JoypadProvider>{
    m_cycle_counter:u32,
    io_bus: IoBus<D, G, J>,
    external_memory_bus:ExternalMemoryBus<'a>,
    oucupied_access_bus:Option<AccessBus>,
    hram: [u8;HRAM_SIZE],
    double_speed_mode:bool,
    mode:Mode,
    #[cfg(feature = "dbg")]
    pub mem_watch:MemoryWatcher,
}


//DMA only locks the used bus. there 2 possible used buses: extrnal (wram, rom, sram) and video (vram)
impl<'a, D:AudioDevice, G:GfxDevice, J:JoypadProvider> Memory for GbMmu<'a, D, G, J>{
    fn read(&mut self, address:u16, m_cycles:u8)->u8{
        #[cfg(feature = "dbg")]
        if self.mem_watch.watching_addrs.as_slice().contains(&address){
            self.mem_watch.hit_addr = Some(address);
        }

        self.cycle(m_cycles);
        if let Some (bus) = &self.oucupied_access_bus{
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
        if self.mem_watch.watching_addrs.as_slice().contains(&address){
            self.mem_watch.hit_addr = Some(address);
        }

        self.cycle(m_cycles);
        if let Some(bus) = &self.oucupied_access_bus{
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
}

impl<'a, D:AudioDevice, G:GfxDevice, J:JoypadProvider> GbMmu<'a, D, G, J>{
    fn read_unprotected(&mut self, address:u16) ->u8 {
        return match address{
            0x0..=0x7FFF=>self.external_memory_bus.read(address),
            0x8000..=0x9FFF=>self.io_bus.ppu.vram.read_current_bank(address-0x8000),
            0xA000..=0xFDFF=>self.external_memory_bus.read(address),
            0xFE00..=0xFE9F=>self.io_bus.ppu.oam[(address-0xFE00) as usize],
            0xFEA0..=0xFEFF=>0x0,
            0xFF00..=0xFF4F | 
            0xFF51..=0xFF6F |
            0xFF71..=0xFF7F=>self.io_bus.read(address - 0xFF00),
            0xFF50 | 0xFF70=>self.external_memory_bus.read(address),
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize],
            0xFFFF=>self.io_bus.interrupt_handler.interrupt_enable_flag
        };
    }

    fn write_unprotected(&mut self, address:u16, value:u8) {
        match address{
            0x0..=0x7FFF=>self.external_memory_bus.write(address, value),
            0x8000..=0x9FFF=>self.io_bus.ppu.vram.write_current_bank(address-0x8000, value),
            0xA000..=0xFDFF=>self.external_memory_bus.write(address, value),
            0xFE00..=0xFE9F=>self.io_bus.ppu.oam[(address-0xFE00) as usize] = value,
            0xFEA0..=0xFEFF=>{},
            0xFF00..=0xFF4F | 
            0xFF51..=0xFF7F=>self.io_bus.write(address - 0xFF00, value),
            0xFF50=>self.external_memory_bus.write(address, value),
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize] = value,
            0xFFFF=>self.io_bus.interrupt_handler.interrupt_enable_flag = value
        }
    }
}

impl<'a, D:AudioDevice, G:GfxDevice, J:JoypadProvider> GbMmu<'a, D, G, J>{
    pub fn new(mbc:&'a mut dyn Mbc, boot_rom:Bootrom, apu:GbApu<D>, gfx_device:G, joypad_proider:J, mode:Mode)->Self{
        let bootrom_missing = boot_rom == Bootrom::None;
        let mut mmu = GbMmu{
            io_bus:IoBus::new(apu, gfx_device, joypad_proider, mode),
            m_cycle_counter:0,
            external_memory_bus: ExternalMemoryBus::new(mbc, boot_rom),
            oucupied_access_bus:None,
            hram:[0;HRAM_SIZE],
            double_speed_mode:false,
            mode,
            #[cfg(feature = "dbg")]
            mem_watch: MemoryWatcher { watching_addrs: crate::utils::FixedSizeSet::new(), hit_addr: None, } 
        };
        if bootrom_missing{
            //Setting the bootrom register to be set (the boot sequence has over)
            mmu.write(BOOT_REGISTER_ADDRESS, 1, 0);
        }

        return mmu;
    }

    pub fn cycle(&mut self, m_cycles:u8){
        flip_bit_u8(&mut self.io_bus.speed_switch_register, 7, self.double_speed_mode);
        self.oucupied_access_bus = self.io_bus.cycle(m_cycles as u32, self.double_speed_mode, &mut self.external_memory_bus);
        self.m_cycle_counter += m_cycles as u32;
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

    pub fn is_frame_finished(&mut self) -> bool{
        if self.m_cycle_counter < CYCLES_PER_FRAME{
            return false;
        }
        self.m_cycle_counter = 0;
        return true;
    }

    #[cfg(feature = "dbg")]
    pub fn get_ppu(&self)->&crate::ppu::gb_ppu::GbPpu<G>{&self.io_bus.ppu}

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
}