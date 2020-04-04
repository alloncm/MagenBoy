use crate::cpu::gbc_cpu::GbcCpu;
use crate::mmu::memory::Memory;
use crate::mmu::gbc_mmu::GbcMmu;
use crate::ppu::gbc_ppu::GbcPpu;
use crate::opcodes::opcodes_utils::*;
use crate::utils::memory_registers::*;




const DMA_SIZE:u16 = 0xA0;
const DMA_DEST:u16 = 0xFE00;

pub fn update_registers_state(memory: &mut GbcMmu, cpu:&mut GbcCpu, ppu:&mut GbcPpu, current_cycle:u32){
    handle_lcdcontrol_register(memory.read(LCDC_REGISTER_ADDRESS), memory, ppu);
    handle_lcdstatus_register(memory.read(STAT_REGISTER_ADDRESS), memory);
    handle_scroll_registers(memory.read(SCX_REGISTER_ADDRESS), memory.read(SCY_REGISTER_ADDRESS), ppu);
    handle_vrambank_register(memory.read(VBK_REGISTER_ADDRESS), memory, cpu);
    handle_switch_mode_register(memory.read(KEYI_REGISTER_ADDRESS), memory, cpu);
    handle_wrambank_register(memory.read(SVBK_REGISTER_ADDRESS), memory);
    handle_dma_transfer_register(memory.read(DMA_REGISTER_ADDRESS), memory);
    handle_bootrom_register(memory.read(BOOT_REGISTER_ADDRESS), memory);
    handle_ly_register(memory, current_cycle);
}

fn handle_ly_register(memory:&mut dyn Memory, current_cycle:u32){
    const C:u32 = 114;
    let mut value = current_cycle/C;
    if value>153{
        value = 153;
    }
    
    memory.write(LY_REGISTER_ADDRESS, value as u8);
}

fn handle_bootrom_register(register:u8, memory: &mut GbcMmu){
    memory.finished_boot = register == 1;
}

fn handle_lcdcontrol_register( register:u8, memory: &mut dyn Memory, ppu:&mut GbcPpu){
    ppu.screen_enable = (register & BIT_7_MASK) != 0;
    ppu.window_tile_map_address = (register & BIT_6_MASK) != 0;
    ppu.window_enable = (register & BIT_5_MASK) != 0;
    ppu.window_tile_background_map_data_address = (register & BIT_4_MASK) != 0;
    ppu.background_tile_map_address = (register & BIT_3_MASK) != 0;
    ppu.sprite_extended = (register & BIT_2_MASK) != 0;
    ppu.sprite_enable = (register & BIT_1_MASK) != 0;
    ppu.background_enabled = (register & BIT_0_MASK) != 0;

    //updates ly register
    if register & BIT_7_MASK == 0{
        memory.write(LY_REGISTER_ADDRESS,0);
    }
}

fn handle_lcdstatus_register( register:u8, memory: &mut dyn Memory){
    let mut coincidence:u8 = (memory.read(LY_REGISTER_ADDRESS) == memory.read(LYC_REGISTER_ADDRESS)) as u8;
    //to match the 2 bit
    coincidence <<=2;
    memory.write(STAT_REGISTER_ADDRESS, register | coincidence);
}

fn handle_scroll_registers(scroll_x:u8, scroll_y:u8, ppu:&mut GbcPpu){
    ppu.background_scroll.x = scroll_x;
    ppu.background_scroll.y = scroll_y;
}

fn handle_vrambank_register( register:u8, memory: &mut GbcMmu, cpu:&mut GbcCpu){
    if cpu.cgb_mode{
        memory.vram.set_bank(register & BIT_0_MASK);
    }
}

fn handle_switch_mode_register( register:u8, memory: &mut dyn Memory, cpu:&mut GbcCpu){
    if register & BIT_0_MASK != 0{
        cpu.cgb_mode = !cpu.cgb_mode;
        let cgb_mask = (cpu.cgb_mode as u8) <<7;
        memory.write(0xFF4D, register | cgb_mask);
    }
}

fn handle_wrambank_register( register:u8, memory: &mut GbcMmu){
    let bank:u8 = register & 0b00000111;
    memory.ram.set_bank(bank);
}

fn handle_dma_transfer_register(register:u8, mmu:&mut GbcMmu){
    if mmu.dma_trasfer_trigger{
        let mut source:u16 = (register as u16) << 8;
        for i in 0..DMA_SIZE{
            source+=1;
            mmu.write(DMA_DEST+i, mmu.read(source));
        }

        mmu.dma_trasfer_trigger = false;
    }
}