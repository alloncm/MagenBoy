use super::{io_components::IoComponents, memory::*};
use super::access_bus::AccessBus;
use crate::{apu::{audio_device::AudioDevice, gb_apu::GbApu}, utils::memory_registers::BOOT_REGISTER_ADDRESS};
use super::carts::mbc::Mbc;
use crate::ppu::ppu_state::PpuState;
use std::boxed::Box;

pub const BOOT_ROM_SIZE:usize = 0x100;
const HRAM_SIZE:usize = 0x7F;
const DMA_SIZE:u16 = 0xA0;
const DMA_DEST:u16 = 0xFE00;

const BAD_READ_VALUE:u8 = 0xFF;

pub struct GbMmu<'a, D:AudioDevice>{
    pub io_components: IoComponents<D>,
    boot_rom:[u8;BOOT_ROM_SIZE],
    mbc: &'a mut Box<dyn Mbc>,
    hram: [u8;HRAM_SIZE],
    interupt_enable_register:u8
}


//DMA only locks the used bus. there 2 possible used buses: extrnal (wram, rom, sram) and video (vram)
impl<'a, D:AudioDevice> Memory for GbMmu<'a, D>{
    fn read(&self, address:u16)->u8{
        if let Some (bus) = &self.io_components.dma.enable{
            return match address{
                0xFF00..=0xFF7F => self.io_components.read(address - 0xFF00),
                0xFEA0..=0xFEFF | 0xFF80..=0xFFFE | 0xFFFF=>self.read_unprotected(address),
                0x8000..=0x9FFF => if let AccessBus::External = bus {self.read_unprotected(address)} else{Self::bad_dma_read(address)},
                0..=0x7FFF | 0xA000..=0xFDFF => if let AccessBus::Video = bus {self.read_unprotected(address)} else{Self::bad_dma_read(address)},
                _=>Self::bad_dma_read(address)
            };
        }
        return match address{
            0x8000..=0x9FFF=>{
                if self.is_vram_ready_for_io(){
                    return self.io_components.ppu.vram.read_current_bank(address-0x8000);
                }
                else{
                    log::warn!("bad vram read");
                    return BAD_READ_VALUE;
                }
            },
            0xFE00..=0xFE9F=>{
                if self.is_oam_ready_for_io(){
                    return self.io_components.ppu.sprite_attribute_table[(address-0xFE00) as usize];
                }
                else{
                    log::warn!("bad oam read");
                    return BAD_READ_VALUE;
                }
            },
            0xFF00..=0xFF7F => self.io_components.read(address - 0xFF00),
            _=>self.read_unprotected(address)
        };
    }

    fn write(&mut self, address:u16, value:u8){
        if let Some(bus) = &self.io_components.dma.enable{
            match address{
                0xFF00..=0xFF7F => self.io_components.write(address- 0xFF00, value),
                0xFF80..=0xFFFE | 0xFFFF=>self.write_unprotected(address, value),
                0x8000..=0x9FFF => if let AccessBus::External = bus {self.write_unprotected(address, value)} else{Self::bad_dma_write(address)},
                0..=0x7FFF | 0xA000..=0xFDFF => if let AccessBus::Video = bus {self.write_unprotected(address, value)} else{Self::bad_dma_write(address)},
                _=>Self::bad_dma_write(address)
            }
        }
        else{
            match address{
                0x8000..=0x9FFF=>{
                    if self.is_vram_ready_for_io(){
                        self.io_components.ppu.vram.write_current_bank(address-0x8000, value);
                    }
                    else{
                        log::warn!("bad vram write")
                    }
                },
                0xFE00..=0xFE9F=>{
                    if self.is_oam_ready_for_io(){
                        self.io_components.ppu.sprite_attribute_table[(address-0xFE00) as usize] = value;
                    }
                    else{
                        log::warn!("bad oam write")
                    }
                },
                0xFF00..=0xFF7F=>self.io_components.write(address - 0xFF00, value),
                _=>self.write_unprotected(address, value)
            }
        }
    }
}

impl<'a, D:AudioDevice> UnprotectedMemory for GbMmu<'a, D>{
    fn read_unprotected(&self, address:u16) ->u8 {
        return match address{
            0x0..=0xFF=>{
                if self.io_components.finished_boot{
                    return self.mbc.read_bank0(address);
                }
                
                return self.boot_rom[address as usize];
            },
            0x100..=0x3FFF=>self.mbc.read_bank0(address),
            0x4000..=0x7FFF=>self.mbc.read_current_bank(address-0x4000),
            0x8000..=0x9FFF=>self.io_components.ppu.vram.read_current_bank(address-0x8000),
            0xA000..=0xBFFF=>self.mbc.read_external_ram(address-0xA000),
            0xC000..=0xCFFF =>self.io_components.ram.read_bank0(address - 0xC000), 
            0xD000..=0xDFFF=>self.io_components.ram.read_current_bank(address-0xD000),
            0xE000..=0xFDFF=>self.io_components.ram.read_bank0(address - 0xE000),
            0xFE00..=0xFE9F=>self.io_components.ppu.sprite_attribute_table[(address-0xFE00) as usize],
            0xFEA0..=0xFEFF=>0x0,
            0xFF00..=0xFF7F=>self.io_components.read_unprotected(address - 0xFF00),
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize],
            0xFFFF=>self.interupt_enable_register
        };
    }

    fn write_unprotected(&mut self, address:u16, value:u8) {
        match address{
            0x0..=0x7FFF=>self.mbc.write_rom(address, value),
            0x8000..=0x9FFF=>self.io_components.ppu.vram.write_current_bank(address-0x8000, value),
            0xA000..=0xBFFF=>self.mbc.write_external_ram(address-0xA000,value),
            0xC000..=0xCFFF =>self.io_components.ram.write_bank0(address - 0xC000,value), 
            0xE000..=0xFDFF=>self.io_components.ram.write_bank0(address - 0xE000,value),
            0xD000..=0xDFFF=>self.io_components.ram.write_current_bank(address-0xD000,value),
            0xFE00..=0xFE9F=>self.io_components.ppu.sprite_attribute_table[(address-0xFE00) as usize] = value,
            0xFEA0..=0xFEFF=>{},
            0xFF00..=0xFF7F=>self.io_components.write_unprotected(address - 0xFF00, value),
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize] = value,
            0xFFFF=>self.interupt_enable_register = value
        }
    }
}

impl<'a, D:AudioDevice> GbMmu<'a, D>{
    pub fn new_with_bootrom(mbc:&'a mut Box<dyn Mbc>, boot_rom:[u8;BOOT_ROM_SIZE], apu:GbApu<D>)->Self{
        GbMmu{
            io_components:IoComponents::new(apu),
            mbc:mbc,
            hram:[0;HRAM_SIZE],
            interupt_enable_register:0,
            boot_rom:boot_rom,
        }
    }

    pub fn new(mbc:&'a mut Box<dyn Mbc>, apu:GbApu<D>)->Self{
        let mut mmu = GbMmu{
            io_components:IoComponents::new(apu),
            mbc:mbc,
            hram:[0;HRAM_SIZE],
            interupt_enable_register:0,
            boot_rom:[0;BOOT_ROM_SIZE],
        };

        //Setting the bootrom register to be set (the boot sequence has over)
        mmu.io_components.write_unprotected(BOOT_REGISTER_ADDRESS - 0xFF00, 1);
        
        mmu
    }

    pub fn cycle(&mut self, cycles:u8){
        self.handle_dma_trasnfer(cycles);
        self.io_components.cycle(cycles as u32);
    }

    fn handle_dma_trasnfer(&mut self, cycles: u8) {
        if self.io_components.dma.enable.is_some(){
            let cycles_to_run = std::cmp::min(self.io_components.dma.dma_cycle_counter + cycles as u16, DMA_SIZE);
            for i in self.io_components.dma.dma_cycle_counter..cycles_to_run as u16{
                self.write_unprotected(DMA_DEST + i, self.read_unprotected(self.io_components.dma.soure_address + i));
            }

            self.io_components.dma.dma_cycle_counter += cycles as u16;
            if self.io_components.dma.dma_cycle_counter >= DMA_SIZE{
                self.io_components.dma.dma_cycle_counter = 0;
                self.io_components.dma.enable = Option::None;
            }
        }
    }

    fn is_oam_ready_for_io(&self)->bool{
        let ppu_state = self.io_components.ppu.state as u8;
        return ppu_state != PpuState::OamSearch as u8 && ppu_state != PpuState::PixelTransfer as u8
    }

    fn is_vram_ready_for_io(&self)->bool{
        return self.io_components.ppu.state as u8 != PpuState::PixelTransfer as u8;
    }

    fn bad_dma_read(address:u16)->u8{
        log::warn!("bad memory read during dma. {:#X}", address);
        return BAD_READ_VALUE;
    }

    fn bad_dma_write(address:u16){
        log::warn!("bad memory write during dma. {:#X}", address)
    }
}