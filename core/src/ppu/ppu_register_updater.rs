use crate::utils::bit_masks::*;
use super::{color::*, gb_ppu::GbPpu, gfx_device::GfxDevice, ppu_state::PpuState};

const WX_OFFSET:u8 = 7;

pub fn handle_lcdcontrol_register<GFX:GfxDevice>( register:u8, ppu:&mut GbPpu<GFX>){
    if ppu.lcd_control & BIT_7_MASK != 0 && register & BIT_7_MASK == 0{
        ppu.turn_off();
    }
    else if ppu.lcd_control & BIT_7_MASK == 0 && register & BIT_7_MASK != 0{
        ppu.turn_on();
    }
    
    ppu.lcd_control = register;
}

pub fn update_stat_register<GFX:GfxDevice>(register:u8, ppu: &mut GbPpu<GFX>){
    ppu.h_blank_interrupt_request = register & BIT_3_MASK != 0;
    ppu.v_blank_interrupt_request = register & BIT_4_MASK != 0;
    ppu.oam_search_interrupt_request = register & BIT_5_MASK != 0;
    ppu.coincidence_interrupt_request = register & BIT_6_MASK != 0;

    ppu.stat_register &= 0b1000_0111;
    ppu.stat_register |= register & 0b111_1000;
}

pub fn set_scx<GFX:GfxDevice>(ppu: &mut GbPpu<GFX>, value:u8){
    ppu.bg_pos.x = value;
}

pub fn set_scy<GFX:GfxDevice>(ppu:&mut GbPpu<GFX>, value:u8){
    ppu.bg_pos.y = value;
}

pub fn handle_bg_pallet_register(register:u8, pallet:&mut [Color;4], palette_register:&mut u8){
    pallet[0] = get_matching_color(register&0b00000011);
    pallet[1] = get_matching_color((register&0b00001100)>>2);
    pallet[2] = get_matching_color((register&0b00110000)>>4);
    pallet[3] = get_matching_color((register&0b11000000)>>6);
    *palette_register = register;
}

pub fn handle_obp_pallet_register(register:u8, pallet:&mut [Option<Color>;4], palette_register:&mut u8){
    pallet[0] = None;
    pallet[1] = Some(get_matching_color((register&0b00001100)>>2));
    pallet[2] = Some(get_matching_color((register&0b00110000)>>4));
    pallet[3] = Some(get_matching_color((register&0b11000000)>>6));
    *palette_register = register;
}

fn get_matching_color(number:u8)->Color{
    return match number{
        0b00=>WHITE,
        0b01=>LIGHT_GRAY,
        0b10=>DARK_GRAY,
        0b11=>BLACK,
        _=>core::panic!("no macthing color for color number: {}", number)
    };
}

pub fn handle_wy_register<GFX:GfxDevice>(register:u8, ppu:&mut GbPpu<GFX>){
    ppu.window_pos.y = register;
}

pub fn handle_wx_register<GFX:GfxDevice>(register:u8, ppu:&mut GbPpu<GFX>){
    if register < WX_OFFSET{
        ppu.window_pos.x = 0;
    }
    else{
        ppu.window_pos.x = register - WX_OFFSET;
    }
}

pub fn get_wx_register<GFX:GfxDevice>(ppu:&GbPpu<GFX>)->u8{
    // This function is not accurate as it wont return wx between 0-6 (will return them as 7)
    return ppu.window_pos.x + WX_OFFSET;
}

pub fn get_stat<GFX:GfxDevice>(ppu:&GbPpu<GFX>)->u8{
    ppu.stat_register
}

pub fn set_lyc<GFX:GfxDevice>(ppu:&mut GbPpu<GFX>, value:u8){
    ppu.lyc_register = value;
}

pub fn set_bgpi<GFX:GfxDevice>(ppu:&mut GbPpu<GFX>, value:u8){
    ppu.bg_color_pallete_index = value;
}

pub fn get_bgpi<GFX:GfxDevice>(ppu:&GbPpu<GFX>)->u8{
    ppu.bg_color_pallete_index
}

pub fn set_bgpd<GFX:GfxDevice>(ppu:&mut GbPpu<GFX>, value:u8){
    set_cgb_color_data_register(ppu.state, &mut ppu.bg_color_ram, &mut ppu.bg_color_pallete_index, value);
}

pub fn get_bgpd<GFX:GfxDevice>(ppu:&GbPpu<GFX>)->u8{
    ppu.bg_color_ram[(ppu.bg_color_pallete_index & 0b11_1111) as usize]
}


pub fn set_obpi<GFX:GfxDevice>(ppu:&mut GbPpu<GFX>, value:u8){
    // bit 6 is discarded
    ppu.obj_color_pallete_index = value & 0b1011_1111;
}

pub fn get_obpi<GFX:GfxDevice>(ppu:&GbPpu<GFX>)->u8{
    ppu.obj_color_pallete_index
}

pub fn set_obpd<GFX:GfxDevice>(ppu:&mut GbPpu<GFX>, value:u8){
    set_cgb_color_data_register(ppu.state, &mut ppu.obj_color_ram, &mut ppu.obj_color_pallete_index, value);
}

pub fn get_obpd<GFX:GfxDevice>(ppu:&GbPpu<GFX>)->u8{
    ppu.obj_color_ram[(ppu.obj_color_pallete_index & 0b11_1111) as usize]
}

pub fn set_orpi<GFX:GfxDevice>(ppu: &mut GbPpu<GFX>, value:u8){
    ppu.cgb_priority_mode = value & BIT_0_MASK == 0;
}

pub fn get_orpi<GFX:GfxDevice>(ppu: &GbPpu<GFX>) -> u8 {
    ppu.cgb_priority_mode as u8
}

fn set_cgb_color_data_register(ppu_state:PpuState, color_ram:&mut[u8;64], pallete_index_register:&mut u8, value:u8){
    // cant wrtite during pixel trasfer, auto increment still takes effect though
    if ppu_state as u8 != PpuState::PixelTransfer as u8{
        color_ram[(*pallete_index_register & 0b11_1111) as usize] = value;
    }
    else{
        log::warn!("bad color ram write: index - {:#X}, value: - {:#X}", pallete_index_register, value);
    }

    // if bit 7 is set inderement the dest adderess after write
    if (*pallete_index_register & BIT_7_MASK) != 0 {
        // Anding with all the bits except bit 6 to achieve wrap behaviour in case of overflow
        *pallete_index_register = (*pallete_index_register + 1) & 0b1011_1111;
    }
}