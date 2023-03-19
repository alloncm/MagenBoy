use crate::mmu::carts::*;
use super::Mode;

const CARTRIDGE_TYPE_ADDRESS:usize = 0x147;
const CGB_FLAG_ADDRESS:usize = 0x143;

pub fn initialize_mbc(program:Vec<u8>, save_data:Option<Vec<u8>>, mode:Option<Mode>)->Box<dyn Mbc>{
    let mbc_type = program[CARTRIDGE_TYPE_ADDRESS];

    let mbc:Box<dyn Mbc> = match mbc_type{
        0x0|0x8=>Box::new(Rom::new(program,false, None)),
        0x9=>Box::new(Rom::new(program, true, save_data)),
        0x1|0x2=>Box::new(Mbc1::new(program,false, None)),
        0x3=>Box::new(Mbc1::new(program,true, save_data)),
        0x11|0x12=>Box::new(Mbc3::new(program,false,Option::None)),
        0x13=>Box::new(Mbc3::new(program, true, save_data)),
        0x19|0x1A=>Box::new(Mbc5::new(program, false, save_data)),
        0x1B=>Box::new(Mbc5::new(program, true, save_data)),
        _=>std::panic!("not supported cartridge: {:#X}",mbc_type)
    };
    
    let cart_compatibility = mbc.get_compatibility_mode();
    if let Some(mode) = mode{
        if cart_compatibility == CartCompatibility::CGB && mode == Mode::DMG{
            std::panic!("Cart supports only CGB and machine is set to DMG");
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

impl dyn Mbc{
    pub fn get_compatibility_mode(&self)->CartCompatibility{
        let cgb_flag = self.read_bank0(CGB_FLAG_ADDRESS as u16);
        return match cgb_flag{
            0x80 => CartCompatibility::ANY,
            0xC0 => CartCompatibility::CGB, 
            _=>     CartCompatibility::DMG
        };
    }
}