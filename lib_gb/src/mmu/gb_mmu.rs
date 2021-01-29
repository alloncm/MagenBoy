use super::{dma::OamDmaTransferer, memory::*};
use super::ram::Ram;
use super::vram::VRam;
use super::io_ports::IoPorts;
use super::access_bus::AccessBus;
use crate::utils::memory_registers::BOOT_REGISTER_ADDRESS;
use super::carts::mbc::Mbc;
use crate::ppu::ppu_state::PpuState;
use std::boxed::Box;

pub const BOOT_ROM_SIZE:usize = 0x100;
const HRAM_SIZE:usize = 0x7F;
const SPRITE_ATTRIBUTE_TABLE_SIZE:usize = 0xA0;

const BAD_READ_VALUE:u8 = 0xFF;

pub struct GbMmu<'a>{
    pub ram: Ram,
    pub vram: VRam,
    pub finished_boot:bool,
    pub io_ports: IoPorts,
    pub dma:OamDmaTransferer,
    boot_rom:[u8;BOOT_ROM_SIZE],
    mbc: &'a mut Box<dyn Mbc>,
    sprite_attribute_table:[u8;SPRITE_ATTRIBUTE_TABLE_SIZE],
    hram: [u8;HRAM_SIZE],
    interupt_enable_register:u8,
    pub ppu_state:PpuState
}


//DMA only locks the used bus. there 2 possible used buses: extrnal (wram, rom, sram) and video (vram)
impl<'a> Memory for GbMmu<'a>{
    fn read(&self, address:u16)->u8{
        if let Some (bus) = &self.dma.enable{
            return match address{
                0xFEA0..=0xFEFF | 0xFF00..=0xFF7F | 0xFF80..=0xFFFE | 0xFFFF=>self.read_unprotected(address),
                0x8000..=0x9FFF => if let AccessBus::External = bus {self.read_unprotected(address)} else{Self::bad_dma_read(address)},
                0..=0x7FFF | 0xA000..=0xFDFF => if let AccessBus::Video = bus {self.read_unprotected(address)} else{Self::bad_dma_read(address)},
                _=>Self::bad_dma_read(address)
            };
        }
        return match address{
            0x8000..=0x9FFF=>{
                if self.is_vram_ready_for_io(){
                    return self.vram.read_current_bank(address-0x8000);
                }
                else{
                    log::warn!("bad vram read");
                    return BAD_READ_VALUE;
                }
            },
            0xFE00..=0xFE9F=>{
                if self.is_oam_ready_for_io(){
                    return self.sprite_attribute_table[(address-0xFE00) as usize];
                }
                else{
                    log::warn!("bad oam read");
                    return BAD_READ_VALUE;
                }
            },
            _=>self.read_unprotected(address)
        };
    }

    fn write(&mut self, address:u16, value:u8){
        if let Some(bus) = &self.dma.enable{
            match address{
                0xFF00..=0xFF7F | 0xFF80..=0xFFFE | 0xFFFF=>self.write_unprotected(address, value),
                0x8000..=0x9FFF => if let AccessBus::External = bus {self.write_unprotected(address, value)} else{Self::bad_dma_write(address)},
                0..=0x7FFF | 0xA000..=0xFDFF => if let AccessBus::Video = bus {self.write_unprotected(address, value)} else{Self::bad_dma_write(address)},
                _=>Self::bad_dma_write(address)
            }
        }
        else{
            match address{
                0x8000..=0x9FFF=>{
                    if self.is_vram_ready_for_io(){
                        self.vram.write_current_bank(address-0x8000, value);
                    }
                    else{
                        log::warn!("bad vram write")
                    }
                },
                0xFE00..=0xFE9F=>{
                    if self.is_oam_ready_for_io(){
                        self.sprite_attribute_table[(address-0xFE00) as usize] = value;
                    }
                    else{
                        log::warn!("bad oam write")
                    }
                },
                _=>self.write_unprotected(address, value)
            }
        }
    }
}

impl<'a> UnprotectedMemory for GbMmu<'a>{
    fn read_unprotected(&self, address:u16) ->u8 {
        return match address{
            0x0..=0xFF=>{
                if self.finished_boot{
                    return self.mbc.read_bank0(address);
                }
                
                return self.boot_rom[address as usize];
            },
            0x100..=0x3FFF=>self.mbc.read_bank0(address),
            0x4000..=0x7FFF=>self.mbc.read_current_bank(address-0x4000),
            0x8000..=0x9FFF=>self.vram.read_current_bank(address-0x8000),
            0xA000..=0xBFFF=>self.mbc.read_external_ram(address-0xA000),
            0xC000..=0xCFFF =>self.ram.read_bank0(address - 0xC000), 
            0xD000..=0xDFFF=>self.ram.read_current_bank(address-0xD000),
            0xE000..=0xFDFF=>self.ram.read_bank0(address - 0xE000),
            0xFE00..=0xFE9F=>self.sprite_attribute_table[(address-0xFE00) as usize],
            0xFEA0..=0xFEFF=>0x0,
            0xFF00..=0xFF7F=>self.io_ports.read(address - 0xFF00),
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize],
            0xFFFF=>self.interupt_enable_register
        };
    }

    fn write_unprotected(&mut self, address:u16, value:u8) {
        match address{
            0x0..=0x7FFF=>self.mbc.write_rom(address, value),
            0x8000..=0x9FFF=>self.vram.write_current_bank(address-0x8000, value),
            0xA000..=0xBFFF=>self.mbc.write_external_ram(address-0xA000,value),
            0xC000..=0xCFFF =>self.ram.write_bank0(address - 0xC000,value), 
            0xE000..=0xFDFF=>self.ram.write_bank0(address - 0xE000,value),
            0xD000..=0xDFFF=>self.ram.write_current_bank(address-0xD000,value),
            0xFE00..=0xFE9F=>self.sprite_attribute_table[(address-0xFE00) as usize] = value,
            0xFEA0..=0xFEFF=>{},
            0xFF00..=0xFF7F=>self.io_ports.write(address - 0xFF00, value),
            0xFF80..=0xFFFE=>self.hram[(address-0xFF80) as usize] = value,
            0xFFFF=>self.interupt_enable_register = value
        }
    }
}

impl<'a> GbMmu<'a>{
    pub fn new_with_bootrom(mbc:&'a mut Box<dyn Mbc>, boot_rom:[u8;BOOT_ROM_SIZE])->Self{
        GbMmu{
            ram:Ram::default(),
            io_ports:IoPorts::default(),
            mbc:mbc,
            vram:VRam::default(),
            sprite_attribute_table:[0;SPRITE_ATTRIBUTE_TABLE_SIZE],
            hram:[0;HRAM_SIZE],
            interupt_enable_register:0,
            boot_rom:boot_rom,
            finished_boot:false,
            ppu_state:PpuState::OamSearch,
            dma:OamDmaTransferer::default()
        }
    }

    pub fn new(mbc:&'a mut Box<dyn Mbc>)->Self{
        let mut mmu = GbMmu{
            ram:Ram::default(),
            io_ports:IoPorts::default(),
            mbc:mbc,
            vram:VRam::default(),
            sprite_attribute_table:[0;SPRITE_ATTRIBUTE_TABLE_SIZE],
            hram:[0;HRAM_SIZE],
            interupt_enable_register:0,
            boot_rom:[0;BOOT_ROM_SIZE],
            finished_boot:true,
            ppu_state:PpuState::OamSearch,
            dma:OamDmaTransferer::default()
        };

        //Setting the bootrom register to be set (the boot sequence has over)
        mmu.io_ports.write_unprotected(BOOT_REGISTER_ADDRESS - 0xFF00, 1);
        
        mmu
    }

    fn is_oam_ready_for_io(&self)->bool{
        let ppu_state = self.ppu_state as u8;
        return ppu_state != PpuState::OamSearch as u8 && ppu_state != PpuState::PixelTransfer as u8
    }

    fn is_vram_ready_for_io(&self)->bool{
        return self.ppu_state as u8 != PpuState::PixelTransfer as u8;
    }

    fn bad_dma_read(address:u16)->u8{
        log::warn!("bad memory read during dma. {:#X}", address);
        return BAD_READ_VALUE;
    }

    fn bad_dma_write(address:u16){
        log::warn!("bad memory write during dma. {:#X}", address)
    }
}