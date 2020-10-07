use lib_gbc::mmu::carts::*;
use std::boxed::Box;
use std::fs;
use log::info;

const CARTRIDGE_TYPE_ADDRESS:usize = 0x147;
const PROGRAM_SUFFIX:&str = ".gb";
pub const SAVE_SUFFIX:&str = ".sav";

pub fn initialize_mbc(program_name:&String)->Box<dyn Mbc>{

    let program_path = format!("{}{}",program_name,PROGRAM_SUFFIX);
    let program = fs::read(program_path).expect("No program found, notice that function must have a `.gb` suffix");

    let mbc_type = program[CARTRIDGE_TYPE_ADDRESS];

    info!("initializing cartridge of type: {:#X}", mbc_type);
    
    let save_data = try_get_save_data(program_name);
    
    return lib_gbc::machine::mbc_initializer::initialize_mbc(mbc_type, program, save_data);
}

fn try_get_save_data(name:&String)->Option<Vec<u8>>{
    let save_path = format!("{}{}",name, SAVE_SUFFIX);
    match fs::read(save_path){
        Ok(ram)=>Some(ram),
        Err(_)=>None
    }
}

pub fn release_mbc(program_name:&String, mbc: Box<dyn Mbc>){
    if mbc.has_battery(){
        while fs::write(format!("{}{}", program_name, ".sav"), mbc.get_ram()).is_err() {}
        
        info!("saved succesfully");
    }
    else{
        info!("No battery detected, no save data created");
    }
}