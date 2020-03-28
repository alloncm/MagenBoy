use super::mbc1::Mbc1;
use super::mbc::Mbc;
use super::rom::Rom;
use std::boxed::Box;

const CARTRIDGE_TYPE_ADDRESS:usize = 0x147;

pub fn initialize_mbc(program:Vec<u8>)->Box<dyn Mbc>{
    let mbc_type = program[CARTRIDGE_TYPE_ADDRESS];
    return match mbc_type{
        0x00|0x08=>Box::new(Rom::new(program)),
        0x01|0x02=>Box::new(Mbc1::new(program)),
        _=>std::panic!("not supported cartridge: {}",mbc_type)
    }
}