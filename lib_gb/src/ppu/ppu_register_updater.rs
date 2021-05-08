use crate::{apu::audio_device::AudioDevice, mmu::{gb_mmu::GbMmu, io_ports::IO_PORTS_MEMORY_OFFSET, memory::UnprotectedMemory}};
use crate::utils::{memory_registers::*, bit_masks::*};
use super::{ gb_ppu::GbPpu, color::*,  colors::*};

const WX_OFFSET:u8 = 7;

const LCDC_REGISTER_INDEX:usize = (LCDC_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;
const STAT_REGISTER_INDEX:usize = (STAT_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;
const SCX_REGISTER_INDEX:usize = (SCX_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;
const SCY_REGISTER_INDEX:usize = (SCY_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;
const BGP_REGISTER_INDEX:usize = (BGP_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;
const OBP0_REGISTER_INDEX:usize = (OBP0_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;
const OBP1_REGISTER_INDEX:usize = (OBP1_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;
const WX_REGISTER_INDEX:usize = (WX_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;
const WY_REGISTER_INDEX:usize = (WY_REGISTER_ADDRESS - IO_PORTS_MEMORY_OFFSET) as usize;

pub fn update_ppu_regsiters<AD:AudioDevice>(memory:&mut GbMmu<AD>, ppu: &mut GbPpu){
    if memory.io_comps.ports.get_ports_cycle_trigger()[LCDC_REGISTER_INDEX] {
        handle_lcdcontrol_register(memory.read_unprotected(LCDC_REGISTER_ADDRESS), ppu);
    }
    if memory.io_comps.ports.get_ports_cycle_trigger()[STAT_REGISTER_INDEX]{
        update_stat_register(memory.read_unprotected(STAT_REGISTER_ADDRESS), ppu);
    }
    if memory.io_comps.ports.get_ports_cycle_trigger()[SCX_REGISTER_INDEX] || memory.io_comps.ports.get_ports_cycle_trigger()[SCY_REGISTER_INDEX]{
        handle_scroll_registers(memory.read_unprotected(SCX_REGISTER_ADDRESS), memory.read_unprotected(SCY_REGISTER_ADDRESS), ppu);
    }
    if memory.io_comps.ports.get_ports_cycle_trigger()[BGP_REGISTER_INDEX]{
        handle_bg_pallet_register(memory.read_unprotected(BGP_REGISTER_ADDRESS), &mut ppu.bg_color_mapping);
    }
    if memory.io_comps.ports.get_ports_cycle_trigger()[OBP0_REGISTER_INDEX]{
        handle_obp_pallet_register(memory.read_unprotected(OBP0_REGISTER_ADDRESS), &mut ppu.obj_color_mapping0);
    }
    if memory.io_comps.ports.get_ports_cycle_trigger()[OBP1_REGISTER_INDEX]{
        handle_obp_pallet_register(memory.read_unprotected(OBP1_REGISTER_ADDRESS), &mut ppu.obj_color_mapping1);
    }
    if memory.io_comps.ports.get_ports_cycle_trigger()[WY_REGISTER_INDEX]{
        handle_wy_register(memory.read_unprotected(WY_REGISTER_ADDRESS), ppu);
    }
    if memory.io_comps.ports.get_ports_cycle_trigger()[WX_REGISTER_INDEX]{
        handle_wx_register(memory.read_unprotected(WX_REGISTER_ADDRESS), ppu);
    }
}


fn handle_lcdcontrol_register( register:u8, ppu:&mut GbPpu){
    ppu.screen_enable = (register & BIT_7_MASK) != 0;
    ppu.window_tile_map_address = (register & BIT_6_MASK) != 0;
    ppu.window_enable = (register & BIT_5_MASK) != 0;
    ppu.window_tile_background_map_data_address = (register & BIT_4_MASK) != 0;
    ppu.background_tile_map_address = (register & BIT_3_MASK) != 0;
    ppu.sprite_extended = (register & BIT_2_MASK) != 0;
    ppu.sprite_enable = (register & BIT_1_MASK) != 0;
    ppu.background_enabled = (register & BIT_0_MASK) != 0;
}

fn update_stat_register(register:u8, ppu: &mut GbPpu){
    ppu.h_blank_interrupt_request = register & BIT_3_MASK != 0;
    ppu.v_blank_interrupt_request = register & BIT_4_MASK != 0;
    ppu.oam_search_interrupt_request = register & BIT_5_MASK != 0;
    ppu.coincidence_interrupt_request = register & BIT_6_MASK != 0;
}

fn handle_scroll_registers(scroll_x:u8, scroll_y:u8, ppu: &mut GbPpu){
    ppu.background_scroll.x = scroll_x;
    ppu.background_scroll.y = scroll_y;
}

fn handle_bg_pallet_register(register:u8, pallet:&mut [Color;4] ){
    pallet[0] = get_matching_color(register&0b00000011);
    pallet[1] = get_matching_color((register&0b00001100)>>2);
    pallet[2] = get_matching_color((register&0b00110000)>>4);
    pallet[3] = get_matching_color((register&0b11000000)>>6);
}

fn handle_obp_pallet_register(register:u8, pallet:&mut [Option<Color>;4] ){
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

fn handle_wy_register(register:u8, ppu:&mut GbPpu){
    ppu.window_scroll.y = register;
}

fn handle_wx_register(register:u8, ppu:&mut GbPpu){
    if register < WX_OFFSET{
        ppu.window_scroll.x = 0;
    }
    else{
        ppu.window_scroll.x = register - WX_OFFSET;
    }
}
