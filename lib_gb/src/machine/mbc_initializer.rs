use crate::{mmu::carts::*, utils::global_static_alloctor::*};
use super::Mode;

const CARTRIDGE_TYPE_ADDRESS:usize = 0x147;
const CGB_FLAG_ADDRESS:usize = 0x143;

pub fn initialize_mbc(program:&[u8], save_data:Option<&[u8]>, mode:Option<Mode>)->&'static mut dyn Mbc{
    let program_clone:&mut [u8] = static_alloc_array(program.len());
    program_clone.clone_from_slice(program);
    let save_data_clone:Option<&'static mut[u8]> = if let Some(sd) = save_data{
        let static_alloc_array = static_alloc_array(sd.len());
        static_alloc_array.clone_from_slice(&sd);
        Some(static_alloc_array)
    }
    else{None};

    let mbc_type = program[CARTRIDGE_TYPE_ADDRESS];

    let mbc:&'static mut dyn Mbc = match mbc_type{
        0x0|0x8=>static_alloc(Rom::new(program_clone,false, None)),
        0x9=>static_alloc(Rom::new(program_clone, true, save_data_clone)),
        0x1|0x2=>static_alloc(Mbc1::new(program_clone,false, None)),
        0x3=>static_alloc(Mbc1::new(program_clone,true, save_data_clone)),
        0x11|0x12=>static_alloc(Mbc3::new(program_clone,false,Option::None)),
        0x13=>static_alloc(Mbc3::new(program_clone, true, save_data_clone)),
        0x19|0x1A=>static_alloc(Mbc5::new(program_clone, false, save_data_clone)),
        0x1B=>static_alloc(Mbc5::new(program_clone, true, save_data_clone)),
        _=>core::panic!("not supported cartridge: {:#X}",mbc_type)
    };
    
    let cart_compatibility = mbc.get_compatibility_mode();
    if let Some(mode) = mode{
        if cart_compatibility == CartCompatibility::CGB && mode == Mode::DMG{
            core::panic!("Cart supports only CGB and machine is set to DMG");
        }
    }
    log::info!("initialized cartridge of type: {:#X} with compatibility for: {} machine", mbc_type, <CartCompatibility as Into<&str>>::into(cart_compatibility));

    return mbc;
}

#[derive(PartialEq)]
pub enum CartCompatibility{
    DMG,
    CGB,
    ANY,
}

impl From<CartCompatibility> for &str{
    fn from(value:CartCompatibility) -> &'static str{
        match value{
            CartCompatibility::ANY => "ANY",
            CartCompatibility::CGB => "CGB",
            CartCompatibility::DMG => "DMG",
        }
    }
}

impl Into<Mode> for CartCompatibility{
    fn into(self) -> Mode {
        match self{
            CartCompatibility::CGB | 
            CartCompatibility::ANY => Mode::CGB,
            CartCompatibility::DMG => Mode::DMG
        }
    }
}

// for some reason the 'a lifetime is important here for the 
// compiler to accept this call on any Mbc lifetine and not just 'static
impl<'a> dyn Mbc + 'a{
    pub fn get_compatibility_mode(&self)->CartCompatibility{
        let cgb_flag = self.read_bank0(CGB_FLAG_ADDRESS as u16);
        return match cgb_flag{
            0x80 => CartCompatibility::ANY,
            0xC0 => CartCompatibility::CGB, 
            _=>     CartCompatibility::DMG
        };
    }
}