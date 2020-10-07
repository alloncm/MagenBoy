extern crate lib_gbc;

use lib_gbc::mmu::carts::mbc1::Mbc1;
use lib_gbc::mmu::carts::mbc::Mbc;
use lib_gbc::mmu::carts::rom::Rom;
use lib_gbc::mmu::carts::mbc3::Mbc3;
use std::boxed::Box;
use std::fs;

const CARTRIDGE_TYPE_ADDRESS:usize = 0x147;
const PROGRAM_SUFFIX:&str = ".gb";
const SAVE_SUFFIX:&str = ".sav";

pub fn initialize_mbc(program_name:&String)->Box<dyn Mbc>{

    let program_path = format!("{}{}",program_name,PROGRAM_SUFFIX);
    let program = fs::read(program_path).expect("No prgram found, notice that function must have a `.gb` suffix");

    let mbc_type = program[CARTRIDGE_TYPE_ADDRESS];
    return match mbc_type{
        0x0|0x8=>Box::new(Rom::new(program,false, None)),
        0x9=>Box::new(Rom::new(program, true, try_get_save_data(program_name))),
        0x1|0x2=>Box::new(Mbc1::new(program,false, None)),
        0x3=>Box::new(Mbc1::new(program,true, try_get_save_data(program_name))),
        0x11|0x12=>Box::new(Mbc3::new(program,false,Option::None)),
        0x13=>Box::new(Mbc3::new(program, true, try_get_save_data(program_name))),
        _=>std::panic!("not supported cartridge: {}",mbc_type)
    }
}

fn try_get_save_data(name:&String)->Option<Vec<u8>>{
    let save_path = format!("{}{}",name, SAVE_SUFFIX);
    match fs::read(save_path){
        Ok(ram)=>Some(ram),
        Err(_)=>None
    }
}