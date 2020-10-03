use super::carts::mbc1::Mbc1;
use super::carts::mbc::Mbc;
use super::carts::rom::Rom;
use super::carts::mbc3::Mbc3;
use std::boxed::Box;

const CARTRIDGE_TYPE_ADDRESS:usize = 0x147;

pub fn initialize_mbc(program:Vec<u8>)->Box<dyn Mbc>{
    let mbc_type = program[CARTRIDGE_TYPE_ADDRESS];
    return match mbc_type{
        0x0|0x8=>Box::new(Rom::new(program)),
        0x1|0x2=>Box::new(Mbc1::new(program)), // No support for 0x3 (not timer)
        0x11|0x12|0x13=>Box::new(Mbc3::new(program)), //no support for 0x10 and 0xF (no timer)
        _=>std::panic!("not supported cartridge: {}",mbc_type)
    }
}