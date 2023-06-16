use super::gb_timer::GbTimer;

pub fn get_div(timer: &GbTimer)->u8{
    (timer.system_counter >> 8) as u8 
}

pub fn set_tima(timer: &mut GbTimer, value:u8){
    timer.tima_register = value;
    timer.tima_overflow = false;
}

pub fn set_tma(timer: &mut GbTimer, value:u8){
    timer.tma_register = value;
}

pub fn set_tac(timer: &mut GbTimer, value:u8){
    timer.tac_tegister = value & 0b111;
}

//Reset on write
pub fn reset_div(timer: &mut GbTimer){
    timer.system_counter = 0;
}