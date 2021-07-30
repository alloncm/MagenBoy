use crate::utils::{vec2::Vec2, bit_masks::*};
use crate::mmu::vram::VRam;
use crate::ppu::color::Color;
use crate::ppu::colors::*;
use crate::ppu::gfx_device::GfxDevice;
use crate::ppu::{ppu_state::PpuState, sprite_attribute::SpriteAttribute};

use super::bg_fetcher::BGFetcher;
use super::sprite_fetcher::SpriteFetcher;


pub struct FifoPpu<GFX: GfxDevice>{
    gfx_device: GFX,

    pub vram: VRam,
    pub oam:[u8;0xA0],
    t_cycles_passed:u16,
    pub state:PpuState,
    pub lcd_control:u8,
    pub stat_register:u8,
    pub lyc_register:u8,
    pub ly_register:u8,
    pub window_pos:Vec2<u8>,
    pub bg_pos:Vec2<u8>,
    pub bg_color_mapping: [Color; 4],
    pub obj_color_mapping0: [Option<Color>;4],
    pub obj_color_mapping1: [Option<Color>;4],

    screen_buffer: [u32; 160*144],
    push_lcd_buffer:Vec<Color>,
    screen_buffer_index:usize,

    //interrupts
    pub v_blank_interrupt_request:bool,
    pub h_blank_interrupt_request:bool,
    pub oam_search_interrupt_request:bool,
    pub coincidence_interrupt_request:bool,

    bg_fetcher:BGFetcher,
    sprite_fetcher:SpriteFetcher,
    stat_triggered:bool,
    trigger_stat_interrupt:bool,
}

impl<GFX:GfxDevice> FifoPpu<GFX>{

    pub fn new(device:GFX) -> Self {
        

        Self{
            gfx_device: device,
            vram: VRam::default(),
            oam: [0;0xA0],
            stat_register: 0,
            lyc_register: 0,
            lcd_control: 0,
            bg_pos: Vec2::<u8>{x:0, y:0},
            window_pos: Vec2::<u8>{x:0,y:0},
            screen_buffer:[0;160*144],
            bg_color_mapping:[WHITE, LIGHT_GRAY, DARK_GRAY, BLACK],
            obj_color_mapping0: [None, Some(LIGHT_GRAY), Some(DARK_GRAY), Some(BLACK)],
            obj_color_mapping1: [None, Some(LIGHT_GRAY), Some(DARK_GRAY), Some(BLACK)],
            ly_register:0,
            state: PpuState::OamSearch,
            //interrupts
            v_blank_interrupt_request:false, 
            h_blank_interrupt_request:false,
            oam_search_interrupt_request:false, 
            coincidence_interrupt_request:false,
            screen_buffer_index:0, 
            t_cycles_passed:0,
            stat_triggered:false,
            trigger_stat_interrupt:false,
            bg_fetcher:BGFetcher::new(),
            sprite_fetcher:SpriteFetcher::new(),
            push_lcd_buffer:Vec::<Color>::new(),
        }
    }

    pub fn turn_off(&mut self){
        self.screen_buffer_index = 0;
        self.t_cycles_passed = 0;
        unsafe{
            std::ptr::write_bytes(self.screen_buffer.as_mut_ptr(), 0xFF, self.screen_buffer.len());
        }
        self.gfx_device.swap_buffer(&self.screen_buffer);
        self.state = PpuState::Hblank;
        self.ly_register = 0;
        self.stat_triggered = false;
        self.trigger_stat_interrupt = false;
        self.bg_fetcher.reset();
        self.sprite_fetcher.reset();
    }

    pub fn turn_on(&mut self){
        self.state = PpuState::OamSearch;
    }

    pub fn cycle(&mut self, m_cycles:u8, if_register:&mut u8){
        if self.lcd_control & BIT_7_MASK == 0{
            return;
        }

        self.cycle_fetcher(m_cycles, if_register);

        //update stat register
        self.stat_register &= 0b1111_1100; //clear first 2 bits
        self.stat_register |= self.state as u8;

        if self.ly_register == self.lyc_register{
            if self.coincidence_interrupt_request {
                self.trigger_stat_interrupt = true;
            }
            self.stat_register |= BIT_2_MASK;
        }
        else{
            self.stat_register &= !BIT_2_MASK;
        }

        if self.trigger_stat_interrupt{
            if !self.stat_triggered{
                *if_register |= BIT_1_MASK;
                self.stat_triggered = true;
            }
        }
        else{
            self.stat_triggered = false;
        }
        
        self.trigger_stat_interrupt = false;

        for pixel in self.push_lcd_buffer.iter(){
            self.screen_buffer[self.screen_buffer_index] = Self::color_as_uint(&pixel);
            self.screen_buffer_index += 1;
            if self.screen_buffer_index == self.screen_buffer.len(){
                self.gfx_device.swap_buffer(&self.screen_buffer);
                self.screen_buffer_index = 0;
            }
        }

        self.push_lcd_buffer.clear();
    }

    fn cycle_fetcher(&mut self, m_cycles:u8, if_register:&mut u8){
        let sprite_height = if (self.lcd_control & BIT_2_MASK) != 0 {16} else {8};

        for _ in 0..m_cycles * 2{
            match self.state{
                PpuState::OamSearch=>{
                    let oam_index = self.t_cycles_passed / 2;
                    let oam_entry_address = (oam_index * 4) as usize;
                    let end_y = self.oam[oam_entry_address];
                    let end_x = self.oam[oam_entry_address + 1];
                
                    if end_x > 0 && self.ly_register + 16 >= end_y && self.ly_register + 16 < end_y + sprite_height && self.sprite_fetcher.oam_entries_len < 10{
                        let tile_number = self.oam[oam_entry_address + 2];
                        let attributes = self.oam[oam_entry_address + 3];
                        self.sprite_fetcher.oam_entries[self.sprite_fetcher.oam_entries_len as usize] = SpriteAttribute::new(end_y, end_x, tile_number, attributes);
                        self.sprite_fetcher.oam_entries_len += 1;
                    }
                    
                    self.t_cycles_passed += 2; //half a m_cycle
                    
                    if self.t_cycles_passed == 80{
                        let slice = self.sprite_fetcher.oam_entries[0..self.sprite_fetcher.oam_entries_len as usize].as_mut();
                        slice.sort_by(|s1:&SpriteAttribute, s2:&SpriteAttribute| s1.x.cmp(&s2.x));
                        self.state = PpuState::PixelTransfer;
                    }
                }
                PpuState::Hblank=>{
                    self.t_cycles_passed += 2;
                    
                    if self.t_cycles_passed == 456{
                        if self.ly_register == 143{
                            self.state = PpuState::Vblank;
                            //reseting the window counter on vblank
                            self.bg_fetcher.window_line_counter = 0;
                            *if_register |= BIT_0_MASK;
                            if self.v_blank_interrupt_request{
                                self.trigger_stat_interrupt = true;
                            }
                        }
                        else{
                            self.state = PpuState::OamSearch;
                            if self.oam_search_interrupt_request{
                                self.trigger_stat_interrupt = true;
                            }
                        }
                        self.t_cycles_passed = 0;
                        self.ly_register += 1;
                    }
                }
                PpuState::Vblank=>{
                    if self.t_cycles_passed == 4560{
                        self.state = PpuState::OamSearch;
                        if self.oam_search_interrupt_request{
                            self.trigger_stat_interrupt = true;
                        }
                        self.t_cycles_passed = 0;
                        self.ly_register = 0;
                    }
                    else{
                        self.ly_register = 144 + (self.t_cycles_passed / 456) as u8;
                    }
                    
                    self.t_cycles_passed += 2;
                }
                PpuState::PixelTransfer=>{
                    if self.bg_fetcher.current_x_pos < 160{
                        self.bg_fetcher.fetch_pixels(&self.vram, self.lcd_control, self.ly_register, &self.window_pos, &self.bg_pos);
                        if self.lcd_control & BIT_1_MASK != 0{
                            self.sprite_fetcher.fetch_pixels(&self.vram, self.ly_register, self.bg_fetcher.current_x_pos);
                        }
                    }
                    
                    for _ in 0..2{
                        self.try_push_to_lcd();
                    }

                    if self.bg_fetcher.current_x_pos == 160 && self.bg_fetcher.fifo.is_empty(){
                        self.state = PpuState::Hblank;
                        if self.h_blank_interrupt_request{
                            self.trigger_stat_interrupt = true;
                        }
                        self.bg_fetcher.try_increment_window_counter();
                        self.bg_fetcher.reset();
                        self.sprite_fetcher.reset();
                    }
                    self.t_cycles_passed += 2;
                }
            }
        }
    }

    fn try_push_to_lcd(&mut self){
        if !self.bg_fetcher.fifo.is_empty(){
            let bg_pixel = self.bg_color_mapping[self.bg_fetcher.fifo.remove(0) as usize];
            let pixel = if !self.sprite_fetcher.fifo.is_empty(){
                let sprite_color_num = self.sprite_fetcher.fifo.remove(0);
                let pixel_oam_attribute = &self.sprite_fetcher.oam_entries[sprite_color_num.1 as usize];

                if sprite_color_num.0 == 0 || pixel_oam_attribute.is_bg_priority{
                    bg_pixel
                }
                else{
                    let sprite_pixel = if pixel_oam_attribute.palette_number{
                        self.obj_color_mapping1[sprite_color_num.0 as usize]
                    }
                    else{
                        self.obj_color_mapping0[sprite_color_num.0 as usize]
                    };

                    if let Some(color) = sprite_pixel{
                        color
                    }
                    else{
                        std::panic!("Corruption in the object color pallete");
                    }
                }
            }
            else{
                bg_pixel
            };

            self.push_lcd_buffer.push(pixel);
        }
    }

    fn color_as_uint(color: &Color) -> u32 {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }
}