use crate::{mmu::carts::*, utils::global_static_alloctor::*};

const CARTRIDGE_TYPE_ADDRESS:usize = 0x147;

pub fn initialize_mbc(program:&[u8], save_data:Option<&[u8]>)->&'static mut dyn Mbc{
    let program_clone:&mut [u8] = static_alloc_array(program.len());
    program_clone.clone_from_slice(program);
    let save_data_clone:Option<&'static mut[u8]> = if let Some(sd) = save_data{
        log::info!("Found save data!");
        let static_alloc_array = static_alloc_array(sd.len());
        static_alloc_array.clone_from_slice(&sd);
        Some(static_alloc_array)
    }
    else{None};

    let mbc_type = program[CARTRIDGE_TYPE_ADDRESS];

    let mbc:&'static mut dyn Mbc = match mbc_type{
        0x0 | 
        0x8 => static_alloc(Rom::new(program_clone,false, None)),
        0x9 => static_alloc(Rom::new(program_clone, true, save_data_clone)),
        0x1 | 
        0x2 => static_alloc(Mbc1::new(program_clone,false, None)),
        0x3 => static_alloc(Mbc1::new(program_clone,true, save_data_clone)),
        0xF => static_alloc(Mbc3::new(program_clone, true, None)),  // The battery is for the RTC which isnt supported right now
        0x10 |  // The battery is also used for the RTC which isnt supported right now
        0x13 => static_alloc(Mbc3::new(program_clone, true, save_data_clone)),
        0x11 | 
        0x12 => static_alloc(Mbc3::new(program_clone,false,None)),
        0x19 | 
        0x1A => static_alloc(Mbc5::new(program_clone, false, save_data_clone)),
        0x1B => static_alloc(Mbc5::new(program_clone, true, save_data_clone)),
        _=> core::panic!("not supported cartridge: {:#X}",mbc_type)
    };
    
    let cart_compatibility = mbc.read_bank0(CGB_FLAG_ADDRESS as u16);
    log::info!("initialized cartridge of type: {:#X} with compatibility {:#X}", mbc_type, cart_compatibility);

    return mbc;
}