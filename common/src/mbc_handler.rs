use magenboy_core::machine::Mode;
use magenboy_core::mmu::carts::*;
use std::fs;
use log::info;

pub const SAVE_SUFFIX:&str = ".sav";

pub fn initialize_mbc(program_name:&String, mode:Option<Mode>)->&'static mut dyn Mbc{
    let program = fs::read(program_name).expect(format!("No program found - {}\n", program_name).as_str());
    let save_data = try_get_save_data(program_name);
    let save_data = if let Some(sd) = &save_data{Some(&sd[..])}else{None};
    return magenboy_core::machine::mbc_initializer::initialize_mbc(&program, save_data, mode);
}

fn try_get_save_data(name:&String)->Option<Vec<u8>>{
    let save_path = format!("{}{}",name, SAVE_SUFFIX);
    match fs::read(save_path){
        Ok(ram)=>Some(ram),
        Err(_)=>None
    }
}

pub fn release_mbc<'a>(program_name:&String, mbc: &'a dyn Mbc){
    if mbc.has_battery(){
        while fs::write(format!("{}{}", program_name, ".sav"), mbc.get_ram()).is_err() {}       
        info!("saved succesfully");
    }
    else{
        info!("No battery detected, no save data created");
    }
}