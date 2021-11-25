use crate::utils::{vec2::Vec2, bit_masks::*};
use crate::mmu::vram::VRam;
use crate::ppu::color::*;
use crate::ppu::colors::*;
use crate::ppu::gfx_device::GfxDevice;
use crate::ppu::{ppu_state::PpuState, sprite_attribute::SpriteAttribute};

use super::fifo::background_fetcher::BackgroundFetcher;
use super::fifo::{FIFO_SIZE, sprite_fetcher::*};

pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_WIDTH: usize = 160;
pub const BUFFERS_NUMBER:usize = 2;

const OAM_ENTRY_SIZE:u16 = 4;
const OAM_MEMORY_SIZE:usize = 0xA0;

const OAM_SEARCH_T_CYCLES_LENGTH: u16 = 80;
const HBLANK_T_CYCLES_LENGTH: u16 = 456;
const VBLANK_T_CYCLES_LENGTH: u16 = 4560;

pub struct GbPpu<GFX: GfxDevice>{
    pub vram: VRam,
    pub oam:[u8;OAM_MEMORY_SIZE],
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

    //interrupts
    pub v_blank_interrupt_request:bool,
    pub h_blank_interrupt_request:bool,
    pub oam_search_interrupt_request:bool,
    pub coincidence_interrupt_request:bool,

    gfx_device: GFX,
    t_cycles_passed:u16,
    screen_buffers: [[u32; SCREEN_HEIGHT * SCREEN_WIDTH];BUFFERS_NUMBER],
    current_screen_buffer_index:usize,
    push_lcd_buffer:Vec<Color>,
    screen_buffer_index:usize,
    pixel_x_pos:u8,
    scanline_started:bool,
    bg_fetcher:BackgroundFetcher,
    sprite_fetcher:SpriteFetcher,
    stat_triggered:bool,
    trigger_stat_interrupt:bool,
}

impl<GFX:GfxDevice> GbPpu<GFX>{
    pub fn new(device:GFX) -> Self {
        Self{
            gfx_device: device,
            vram: VRam::default(),
            oam: [0;OAM_MEMORY_SIZE],
            stat_register: 0,
            lyc_register: 0,
            lcd_control: 0,
            bg_pos: Vec2::<u8>{x:0, y:0},
            window_pos: Vec2::<u8>{x:0,y:0},
            screen_buffers:[[0;SCREEN_HEIGHT * SCREEN_WIDTH];BUFFERS_NUMBER],
            current_screen_buffer_index:0,
            bg_color_mapping:[WHITE, LIGHT_GRAY, DARK_GRAY, BLACK],
            obj_color_mapping0: [None, Some(LIGHT_GRAY), Some(DARK_GRAY), Some(BLACK)],
            obj_color_mapping1: [None, Some(LIGHT_GRAY), Some(DARK_GRAY), Some(BLACK)],
            ly_register:0,
            state: PpuState::Hblank,
            //interrupts
            v_blank_interrupt_request:false, 
            h_blank_interrupt_request:false,
            oam_search_interrupt_request:false, 
            coincidence_interrupt_request:false,
            screen_buffer_index:0, 
            t_cycles_passed:0,
            stat_triggered:false,
            trigger_stat_interrupt:false,
            bg_fetcher:BackgroundFetcher::new(),
            sprite_fetcher:SpriteFetcher::new(),
            push_lcd_buffer:Vec::<Color>::new(),
            pixel_x_pos:0,
            scanline_started:false
        }
    }

    pub fn turn_off(&mut self){
        self.t_cycles_passed = 0;
        //This is an expensive operation!
        unsafe{std::ptr::write_bytes(self.screen_buffers[self.current_screen_buffer_index].as_mut_ptr(), 0xFF, SCREEN_HEIGHT * SCREEN_WIDTH)};
        self.swap_buffer();
        self.state = PpuState::Hblank;
        self.ly_register = 0;
        self.stat_triggered = false;
        self.trigger_stat_interrupt = false;
        self.bg_fetcher.has_wy_reached_ly = false;
        self.bg_fetcher.window_line_counter = 0;
        self.bg_fetcher.reset();
        self.sprite_fetcher.reset();
        self.pixel_x_pos = 0;
    }

    pub fn turn_on(&mut self){
        self.state = PpuState::OamSearch;
    }

    pub fn cycle(&mut self, m_cycles:u32, if_register:&mut u8){
        if self.lcd_control & BIT_7_MASK == 0{
            return;
        }

        self.cycle_fetcher(m_cycles, if_register);

        self.update_stat_register(if_register);

        for i in 0..self.push_lcd_buffer.len(){
            self.screen_buffers[self.current_screen_buffer_index][self.screen_buffer_index] = u32::from(self.push_lcd_buffer[i]);
            self.screen_buffer_index += 1;
            if self.screen_buffer_index == SCREEN_WIDTH * SCREEN_HEIGHT{
               self.swap_buffer();
            }
        }

        self.push_lcd_buffer.clear();
    }

    fn swap_buffer(&mut self){
        self.gfx_device.swap_buffer(&self.screen_buffers[self.current_screen_buffer_index]);
        self.screen_buffer_index = 0;
        self.current_screen_buffer_index = (self.current_screen_buffer_index + 1) % BUFFERS_NUMBER;
    }

    fn update_stat_register(&mut self, if_register: &mut u8) {
        self.stat_register &= 0b1111_1100;
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
    }

    fn cycle_fetcher(&mut self, m_cycles:u32, if_register:&mut u8){
        let sprite_height = if (self.lcd_control & BIT_2_MASK) != 0 {EXTENDED_SPRITE_HIGHT} else {NORMAL_SPRITE_HIGHT};

        for _ in 0..m_cycles * 2{
            match self.state{
                PpuState::OamSearch=>{
                    let oam_index = self.t_cycles_passed / 2;
                    let oam_entry_address = (oam_index * OAM_ENTRY_SIZE) as usize;
                    let end_y = self.oam[oam_entry_address];
                    let end_x = self.oam[oam_entry_address + 1];
                
                    if end_x > 0 && self.ly_register + 16 >= end_y && self.ly_register + 16 < end_y + sprite_height && self.sprite_fetcher.oam_entries_len < MAX_SPRITES_PER_LINE as u8{
                        let tile_number = self.oam[oam_entry_address + 2];
                        let attributes = self.oam[oam_entry_address + 3];
                        self.sprite_fetcher.oam_entries[self.sprite_fetcher.oam_entries_len as usize] = SpriteAttribute::new(end_y, end_x, tile_number, attributes);
                        self.sprite_fetcher.oam_entries_len += 1;
                    }
                    
                    self.t_cycles_passed += 2; //half a m_cycle
                    
                    if self.t_cycles_passed == OAM_SEARCH_T_CYCLES_LENGTH{
                        let slice = self.sprite_fetcher.oam_entries[0..self.sprite_fetcher.oam_entries_len as usize].as_mut();
                        slice.sort_by(|s1:&SpriteAttribute, s2:&SpriteAttribute| s1.x.cmp(&s2.x));
                        self.state = PpuState::PixelTransfer;
                        self.scanline_started = false;
                    }
                }
                PpuState::Hblank=>{
                    self.t_cycles_passed += 2;
                    
                    if self.t_cycles_passed == HBLANK_T_CYCLES_LENGTH{
                        self.pixel_x_pos = 0;
                        self.t_cycles_passed = 0;
                        self.ly_register += 1;
                        if self.ly_register == SCREEN_HEIGHT as u8{
                            self.state = PpuState::Vblank;
                            //reseting the window counter on vblank
                            self.bg_fetcher.window_line_counter = 0;
                            self.bg_fetcher.has_wy_reached_ly = false;
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
                    }
                }
                PpuState::Vblank=>{
                    if self.t_cycles_passed == VBLANK_T_CYCLES_LENGTH{
                        self.state = PpuState::OamSearch;
                        if self.oam_search_interrupt_request{
                            self.trigger_stat_interrupt = true;
                        }
                        self.pixel_x_pos = 0;
                        self.t_cycles_passed = 0;
                        self.ly_register = 0;
                    }
                    else{
                        //VBlank is technically 10 HBlank combined
                        self.ly_register = SCREEN_HEIGHT as u8 + (self.t_cycles_passed / HBLANK_T_CYCLES_LENGTH) as u8;
                    }
                    
                    self.t_cycles_passed += 2;
                }
                PpuState::PixelTransfer=>{
                    for _ in 0..2{
                        if self.pixel_x_pos < SCREEN_WIDTH as u8{
                            if self.lcd_control & BIT_1_MASK != 0{
                                self.sprite_fetcher.fetch_pixels(&self.vram, self.lcd_control, self.ly_register, self.pixel_x_pos);
                            }
                            if self.sprite_fetcher.rendering{
                                self.bg_fetcher.pause();
                            }
                            else{
                                self.bg_fetcher.fetch_pixels(&self.vram, self.lcd_control, self.ly_register, &self.window_pos, &self.bg_pos);
                                self.try_push_to_lcd();
                                if self.pixel_x_pos == SCREEN_WIDTH as u8{
                                    self.state = PpuState::Hblank;
                                    if self.h_blank_interrupt_request{
                                        self.trigger_stat_interrupt = true;
                                    }
                                    self.bg_fetcher.try_increment_window_counter(self.ly_register, self.window_pos.y);
                                    self.bg_fetcher.reset();
                                    self.sprite_fetcher.reset();

                                    // If im on the first iteration and finished the 160 pixels break;
                                    // In this case the number of t_cycles should be eneven but it will break
                                    // my code way too much for now so Im leaving this as it is... (maybe in the future)
                                    break;
                                }
                            }
                        }
                    }
                    self.t_cycles_passed += 2;
                }
            }
            
        }
    }

    fn try_push_to_lcd(&mut self){
        if !(self.bg_fetcher.fifo.len() == 0){
            if !self.scanline_started{
                // discard the next pixel in the bg fifo
                // the bg fifo should start with 8 pixels and not push more untill its empty again
                if FIFO_SIZE as usize - self.bg_fetcher.fifo.len() >= self.bg_pos.x as usize % FIFO_SIZE as usize{
                    self.scanline_started = true;
                }
                else{
                    self.bg_fetcher.fifo.remove();
                    return;
                }
            }

            let bg_pixel_color_num = self.bg_fetcher.fifo.remove();
            let bg_pixel = self.bg_color_mapping[bg_pixel_color_num as usize];
            let pixel = if !(self.sprite_fetcher.fifo.len() == 0){
                let sprite_color_num = self.sprite_fetcher.fifo.remove();
                let pixel_oam_attribute = &self.sprite_fetcher.oam_entries[sprite_color_num.1 as usize];

                if sprite_color_num.0 == 0 || (pixel_oam_attribute.is_bg_priority && bg_pixel_color_num != 0){
                    bg_pixel
                }
                else{
                    let sprite_pixel = if pixel_oam_attribute.palette_number{
                        self.obj_color_mapping1[sprite_color_num.0 as usize]
                    }
                    else{
                        self.obj_color_mapping0[sprite_color_num.0 as usize]
                    };

                    sprite_pixel.expect("Corruption in the object color pallete")
                }
            }
            else{
                bg_pixel
            };

            self.push_lcd_buffer.push(pixel);
            self.pixel_x_pos += 1;
        }
    }
}