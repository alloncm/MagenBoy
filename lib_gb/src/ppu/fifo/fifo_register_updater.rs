use crate::utils::bit_masks::*;
use crate::ppu::{color::*, colors::*, fifo::fifo_ppu::FifoPpu, gfx_device::GfxDevice};

const WX_OFFSET:u8 = 7;

pub fn handle_lcdcontrol_register<GFX:GfxDevice>( register:u8, ppu:&mut FifoPpu<GFX>){
    if ppu.lcd_control & BIT_7_MASK != 0 && register & BIT_7_MASK == 0{
        ppu.turn_off();
    }
    else if ppu.lcd_control & BIT_7_MASK == 0 && register & BIT_7_MASK != 0{
        ppu.turn_on();
    }
    
    ppu.lcd_control = register;
}

pub fn update_stat_register<GFX:GfxDevice>(register:u8, ppu: &mut FifoPpu<GFX>){
    ppu.h_blank_interrupt_request = register & BIT_3_MASK != 0;
    ppu.v_blank_interrupt_request = register & BIT_4_MASK != 0;
    ppu.oam_search_interrupt_request = register & BIT_5_MASK != 0;
    ppu.coincidence_interrupt_request = register & BIT_6_MASK != 0;

    ppu.stat_register = register & 0b111_1000;
}

pub fn set_scx<GFX:GfxDevice>(ppu: &mut FifoPpu<GFX>, value:u8){
    ppu.bg_pos.x = value;
}

pub fn set_scy<GFX:GfxDevice>(ppu:&mut FifoPpu<GFX>, value:u8){
    ppu.bg_pos.y = value;
}

pub fn handle_bg_pallet_register(register:u8, pallet:&mut [Color;4] ){
    pallet[0] = get_matching_color(register&0b00000011);
    pallet[1] = get_matching_color((register&0b00001100)>>2);
    pallet[2] = get_matching_color((register&0b00110000)>>4);
    pallet[3] = get_matching_color((register&0b11000000)>>6);
}

pub fn handle_obp_pallet_register(register:u8, pallet:&mut [Option<Color>;4] ){
    pallet[0] = None;
    pallet[1] = Some(get_matching_color((register&0b00001100)>>2));
    pallet[2] = Some(get_matching_color((register&0b00110000)>>4));
    pallet[3] = Some(get_matching_color((register&0b11000000)>>6));
}

fn get_matching_color(number:u8)->Color{
    return match number{
        0b00=>WHITE,
        0b01=>LIGHT_GRAY,
        0b10=>DARK_GRAY,
        0b11=>BLACK,
        _=>std::panic!("no macthing color for color number: {}", number)
    };
}

pub fn handle_wy_register<GFX:GfxDevice>(register:u8, ppu:&mut FifoPpu<GFX>){
    ppu.window_pos.y = register;
}

pub fn handle_wx_register<GFX:GfxDevice>(register:u8, ppu:&mut FifoPpu<GFX>){
    if register < WX_OFFSET{
        ppu.window_pos.x = 0;
    }
    else{
        ppu.window_pos.x = register - WX_OFFSET;
    }
}

pub fn get_ly<GFX:GfxDevice>(ppu:&FifoPpu<GFX>)->u8{
    ppu.ly_register
}

pub fn get_stat<GFX:GfxDevice>(ppu:&FifoPpu<GFX>)->u8{
    ppu.stat_register
}

pub fn set_lyc<GFX:GfxDevice>(ppu:&mut FifoPpu<GFX>, value:u8){
    ppu.lyc_register = value;
}