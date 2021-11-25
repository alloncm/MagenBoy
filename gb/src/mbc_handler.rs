use lib_gb::mmu::carts::*;
use std::boxed::Box;
use std::fs;
use log::info;

const PROGRAM_SUFFIX:&str = ".gb";
pub const SAVE_SUFFIX:&str = ".sav";

pub fn initialize_mbc(program_name:&String)->Box<dyn Mbc>{

    let program_path = format!("{}{}",program_name,PROGRAM_SUFFIX);
    let error_message = format!("No program found, notice that the file must have a `.gb` suffix - {}\n", program_name);
    let program = fs::read(program_path).expect(error_message.as_str());
    let save_data = try_get_save_data(program_name);
    
    return lib_gb::machine::mbc_initializer::initialize_mbc(program, save_data);
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