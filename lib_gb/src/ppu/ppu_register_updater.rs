use crate::utils::bit_masks::*;
use super::{ gb_ppu::GbPpu, color::*,  colors::*};

const WX_OFFSET:u8 = 7;

pub fn handle_lcdcontrol_register( register:u8, ppu:&mut GbPpu){
    ppu.screen_enable = (register & BIT_7_MASK) != 0;
    ppu.window_tile_map_address = (register & BIT_6_MASK) != 0;
    ppu.window_enable = (register & BIT_5_MASK) != 0;
    ppu.window_tile_background_map_data_address = (register & BIT_4_MASK) != 0;
    ppu.background_tile_map_address = (register & BIT_3_MASK) != 0;
    ppu.sprite_extended = (register & BIT_2_MASK) != 0;
    ppu.sprite_enable = (register & BIT_1_MASK) != 0;
    ppu.background_enabled = (register & BIT_0_MASK) != 0;
}

pub fn update_stat_register(register:u8, ppu: &mut GbPpu){
    ppu.h_blank_interrupt_request = register & BIT_3_MASK != 0;
    ppu.v_blank_interrupt_request = register & BIT_4_MASK != 0;
    ppu.oam_search_interrupt_request = register & BIT_5_MASK != 0;
    ppu.coincidence_interrupt_request = register & BIT_6_MASK != 0;

    ppu.stat_register = register & 0b111_1000;
}

pub fn handle_scroll_registers(scroll_x:u8, scroll_y:u8, ppu: &mut GbPpu){
    ppu.background_scroll.x = scroll_x;
    ppu.background_scroll.y = scroll_y;
}

pub fn set_scx(ppu: &mut GbPpu, value:u8){
    ppu.background_scroll.x = value;
}

pub fn set_scy(ppu:&mut GbPpu, value:u8){
    ppu.background_scroll.y = value;
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

pub fn handle_wy_register(register:u8, ppu:&mut GbPpu){
    ppu.window_scroll.y = register;
}

pub fn handle_wx_register(register:u8, ppu:&mut GbPpu){
    if register < WX_OFFSET{
        ppu.window_scroll.x = 0;
    }
    else{
        ppu.window_scroll.x = register - WX_OFFSET;
    }
}

pub fn get_ly(ppu:&GbPpu)->u8{
    ppu.current_line_drawn
}

pub fn get_stat(ppu:&GbPpu)->u8{
    ppu.stat_register
}

pub fn set_lyc(ppu:&mut GbPpu, value:u8){
    ppu.lyc_register = value;
}
