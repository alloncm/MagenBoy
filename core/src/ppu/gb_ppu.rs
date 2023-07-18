use core::cmp;

use crate::{machine::Mode, utils::{vec2::Vec2, bit_masks::*}};

use super::{
    fifo::{SPRITE_WIDTH, background_fetcher::*, FIFO_SIZE, sprite_fetcher::*}, 
    VRam, gfx_device::*, ppu_state::PpuState, attributes::SpriteAttributes, color::*
};

pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_WIDTH: usize = 160;
pub const BUFFERS_NUMBER:usize = 2;

const OAM_ENTRY_SIZE:u16 = 4;
const OAM_MEMORY_SIZE:usize = 0xA0;

const OAM_SEARCH_M_CYCLES_LENGTH: u16 = 80 / 4;
const HBLANK_M_CYCLES_LENGTH: u16 = 456 / 4;
const VBLANK_M_CYCLES_LENGTH: u16 = 4560 / 4;

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
    pub bg_palette_register:u8,
    pub bg_color_mapping: [Color; 4],
    pub obj_pallete_0_register:u8,
    pub obj_color_mapping0: [Option<Color>;4],
    pub obj_pallete_1_register:u8,
    pub obj_color_mapping1: [Option<Color>;4],

    // CGB
    pub bg_color_ram:[u8;64],
    pub bg_color_pallete_index:u8,
    pub obj_color_ram:[u8;64],
    pub obj_color_pallete_index:u8,

    //interrupts
    pub v_blank_interrupt_request:bool,
    pub h_blank_interrupt_request:bool,
    pub oam_search_interrupt_request:bool,
    pub coincidence_interrupt_request:bool,

    gfx_device: GFX,
    m_cycles_passed:u16,
    screen_buffers: [[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH];BUFFERS_NUMBER],
    current_screen_buffer_index:usize,
    screen_buffer_index:usize,
    pixel_x_pos:u8,
    scanline_started:bool,
    bg_fetcher:BackgroundFetcher,
    sprite_fetcher:SpriteFetcher,
    stat_triggered:bool,
    trigger_stat_interrupt:bool,
    next_state:PpuState,
    mode: Mode,
}

impl<GFX:GfxDevice> GbPpu<GFX>{
    pub fn new(device:GFX, mode: Mode) -> Self {
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
            bg_palette_register:0,
            bg_color_mapping:[WHITE, LIGHT_GRAY, DARK_GRAY, BLACK],
            obj_pallete_0_register:0,
            obj_color_mapping0: [None, Some(LIGHT_GRAY), Some(DARK_GRAY), Some(BLACK)],
            obj_pallete_1_register:0,
            obj_color_mapping1: [None, Some(LIGHT_GRAY), Some(DARK_GRAY), Some(BLACK)],
            ly_register:0,
            state: PpuState::Hblank,
            // CGB
            bg_color_ram:[0;64],
            bg_color_pallete_index:0,
            obj_color_ram:[0;64],
            obj_color_pallete_index:0,
            //interrupts
            v_blank_interrupt_request:false, 
            h_blank_interrupt_request:false,
            oam_search_interrupt_request:false, 
            coincidence_interrupt_request:false,
            screen_buffer_index:0, 
            m_cycles_passed:0,
            stat_triggered:false,
            trigger_stat_interrupt:false,
            bg_fetcher:BackgroundFetcher::new(mode),
            sprite_fetcher:SpriteFetcher::new(mode),
            pixel_x_pos:0,
            scanline_started:false,
            next_state:PpuState::OamSearch,
            mode,
        }
    }

    pub fn turn_off(&mut self){
        self.m_cycles_passed = 0;
        //This is an expensive operation!
        unsafe{core::ptr::write_bytes(self.screen_buffers[self.current_screen_buffer_index].as_mut_ptr(), 0xFF, SCREEN_HEIGHT * SCREEN_WIDTH)};
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

    #[cfg(feature = "dbg")]
    pub fn get_bg_layer(&self)->[Pixel; 0x100*0x100]{
        use core::convert::TryInto;
        use super::attributes::GbcBackgroundAttributes;

        let bg_tile_map_addr = if self.lcd_control & BIT_3_MASK == 0 {0x1800} else {0x1C00};
        let vram_bank = self.vram.get_current_bank();
        let tile_map:&[u8; 32*32] = vram_bank[bg_tile_map_addr .. bg_tile_map_addr + (32 * 32)].try_into().unwrap();
        let tiles = tile_map.map(|tile_index|{
            let bg_tile_data_addr = if self.lcd_control & BIT_4_MASK == 0 && tile_index & BIT_7_MASK == 0 {0x1000} else {0};
            let tile_addr = bg_tile_data_addr + tile_index as usize * 16;
            let tile_data:&[u8;16] = vram_bank[tile_addr .. tile_addr + 16].try_into().unwrap();
            tile_data.clone()
        });
        let mut buffer = [Pixel::default();0x100*0x100];
        for i in 0 .. (32 * 32){
            let tile_data = &tiles[i];
            for j in 0..8{
                let upper_byte = tile_data[j * 2];
                let lower_byte = tile_data[(j * 2) + 1];
                let y = ((i / 32) * 8) + (j * 0x100);
                for k in 0..8{
                    let x = ((i % 32) * 8) + k;
                    let mask = 1 << k;
                    let pixel = (upper_byte & mask) | (lower_byte & mask);
                    buffer[y + x] = self.get_bg_pixel(BackgroundPixel { color_index: pixel, attributes:  GbcBackgroundAttributes::new(0)}).into();
                }
            }
        }
        return buffer;
    }

    pub fn cycle(&mut self, m_cycles:u32, if_register:&mut u8)->Option<u32>{
        if self.lcd_control & BIT_7_MASK == 0{
            return None;
        }

        let fethcer_m_cycles_to_next_event = self.cycle_fetcher(m_cycles, if_register) as u32;

        let stat_m_cycles_to_next_event = self.update_stat_register(if_register);

        let cycles = cmp::min(fethcer_m_cycles_to_next_event, stat_m_cycles_to_next_event);

        return Some(cycles);
    }

    fn swap_buffer(&mut self){
        self.gfx_device.swap_buffer(&self.screen_buffers[self.current_screen_buffer_index]);
        self.screen_buffer_index = 0;
        self.current_screen_buffer_index = (self.current_screen_buffer_index + 1) % BUFFERS_NUMBER;
    }

    fn update_stat_register(&mut self, if_register: &mut u8) -> u32{
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

        let t_cycles_to_next_stat_change = if self.lyc_register < self.ly_register{
            ((self.ly_register - self.lyc_register) as u32 * HBLANK_M_CYCLES_LENGTH as u32) - self.m_cycles_passed as u32
        }
        else if self.lyc_register == self.ly_register{
            (HBLANK_M_CYCLES_LENGTH as u32 * 154 ) - self.m_cycles_passed as u32
        }
        else{
            ((self.lyc_register - self.ly_register) as u32 * HBLANK_M_CYCLES_LENGTH as u32) - (self.m_cycles_passed as u32 % HBLANK_M_CYCLES_LENGTH as u32)
        };

        return t_cycles_to_next_stat_change;
    }

    fn cycle_fetcher(&mut self, m_cycles:u32, if_register:&mut u8)->u16{
        let mut m_cycles_counter = 0;

        while m_cycles_counter < m_cycles{
            match self.next_state{
                PpuState::OamSearch=>{
                    self.state = PpuState::OamSearch;
                    // first iteration
                    if self.m_cycles_passed == 0{
                        self.read_sprites_from_oam();
                    }
                    
                    let scope_m_cycles_passed = cmp::min(m_cycles as u16, OAM_SEARCH_M_CYCLES_LENGTH - self.m_cycles_passed);
                    self.m_cycles_passed += scope_m_cycles_passed;
                    m_cycles_counter += scope_m_cycles_passed as u32;
                    
                    if self.m_cycles_passed == OAM_SEARCH_M_CYCLES_LENGTH{
                        self.next_state = PpuState::PixelTransfer;
                        self.scanline_started = false;
                    }
                }
                PpuState::Hblank=>{
                    self.state = PpuState::Hblank;
                    let m_cycles_to_add = cmp::min((m_cycles - m_cycles_counter) as u16, HBLANK_M_CYCLES_LENGTH - self.m_cycles_passed);
                    self.m_cycles_passed += m_cycles_to_add;
                    m_cycles_counter += m_cycles_to_add as u32;
                    
                    if self.m_cycles_passed == HBLANK_M_CYCLES_LENGTH{
                        self.pixel_x_pos = 0;
                        self.m_cycles_passed = 0;
                        self.ly_register += 1;
                        if self.ly_register == SCREEN_HEIGHT as u8{
                            self.next_state = PpuState::Vblank;
                            //reseting the window counter on vblank
                            self.bg_fetcher.window_line_counter = 0;
                            self.bg_fetcher.has_wy_reached_ly = false;
                            *if_register |= BIT_0_MASK;
                            if self.v_blank_interrupt_request{
                                self.trigger_stat_interrupt = true;
                            }
                        }
                        else{
                            self.next_state = PpuState::OamSearch;
                            if self.oam_search_interrupt_request{
                                self.trigger_stat_interrupt = true;
                            }
                        }
                    }
                }
                PpuState::Vblank=>{
                    self.state = PpuState::Vblank;
                    let m_cycles_to_add = cmp::min((m_cycles - m_cycles_counter) as u16, VBLANK_M_CYCLES_LENGTH - self.m_cycles_passed);
                    self.m_cycles_passed += m_cycles_to_add;
                    m_cycles_counter += m_cycles_to_add as u32;
                    
                    if self.m_cycles_passed == VBLANK_M_CYCLES_LENGTH{
                        self.next_state = PpuState::OamSearch;
                        if self.oam_search_interrupt_request{
                            self.trigger_stat_interrupt = true;
                        }
                        self.pixel_x_pos = 0;
                        self.m_cycles_passed = 0;
                        self.ly_register = 0;
                    }
                    else{
                        //VBlank is technically 10 HBlank combined
                        self.ly_register = SCREEN_HEIGHT as u8 + (self.m_cycles_passed / HBLANK_M_CYCLES_LENGTH) as u8;
                    }
                    
                }
                PpuState::PixelTransfer=>{
                    self.state = PpuState::PixelTransfer;
                    while m_cycles_counter < m_cycles && self.pixel_x_pos < SCREEN_WIDTH as u8{
                        for _ in 0..4{
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
                                    self.next_state = PpuState::Hblank;
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

                        self.m_cycles_passed += 1;
                        m_cycles_counter += 1;
                    }
                }
            }
        }

        // If there was a state change I want to run the state machine another m_cycle
        // in order to sync the stat register containing the ppu state (by allowing state and next_state to sync)
        if self.next_state as u8 != self.state as u8{
            return 1;
        }
        let m_cycles_for_state = match self.next_state{
            PpuState::Vblank => ((self.m_cycles_passed / HBLANK_M_CYCLES_LENGTH)+1) * HBLANK_M_CYCLES_LENGTH,
            PpuState::Hblank => HBLANK_M_CYCLES_LENGTH,
            PpuState::OamSearch => OAM_SEARCH_M_CYCLES_LENGTH,
            
            // taking the pixels that left to draw and divide by 4 (usually pushing 4 pixels per m_cycle) 
            // to try and calculate how much cycles left for the pixel transfer state
            PpuState::PixelTransfer => self.m_cycles_passed + ((SCREEN_WIDTH - self.pixel_x_pos as usize) as u16 >> 2) 
        };

        return m_cycles_for_state - self.m_cycles_passed;
    }

    fn read_sprites_from_oam(&mut self) {
        let sprite_height = if (self.lcd_control & BIT_2_MASK) != 0 {EXTENDED_SPRITE_HIGHT} else {NORMAL_SPRITE_HIGHT};
        for oam_index in 0..(OAM_MEMORY_SIZE as u16 / OAM_ENTRY_SIZE){
            let oam_entry_address = (oam_index * OAM_ENTRY_SIZE) as usize;
            let end_y = self.oam[oam_entry_address];
            let end_x = self.oam[oam_entry_address + 1];

            if end_x > 0 && end_x < SCREEN_WIDTH as u8 + SPRITE_WIDTH && self.ly_register + 16 >= end_y && self.ly_register + 16 < end_y + sprite_height {
                let tile_number = self.oam[oam_entry_address + 2];
                let attributes = self.oam[oam_entry_address + 3];
                let mut vis_start = 0;
                let mut vis_end = SPRITE_WIDTH;
                for i in 0..self.sprite_fetcher.oam_entries_len{
                    let mut entry = &mut self.sprite_fetcher.oam_entries[i as usize];
                    // check collision
                    if entry.x < end_x + SPRITE_WIDTH && entry.x + SPRITE_WIDTH > end_x {
                        // Use min/max to get the lowest/highest end/start point and making sure it wont get override by later entries
                        // TODO: check iterating over the oam_entries in reverse so that index 0 (the highest prioirty on CGB is last)
                        if end_x < entry.x{
                            if self.mode == Mode::CGB{
                                vis_end = cmp::min(entry.x - end_x, vis_end);
                            }else{
                                entry.visibility_start = cmp::max(SPRITE_WIDTH - (entry.x - end_x), entry.visibility_start);
                            }
                        }else if end_x > entry.x{
                            vis_start = cmp::max(SPRITE_WIDTH - (end_x - entry.x), vis_start);
                        }else{
                            vis_start = SPRITE_WIDTH;
                        }
                    }
                }
        
                // bounds checks
                if end_x < SPRITE_WIDTH{
                    vis_start = cmp::max(SPRITE_WIDTH - end_x, vis_start);
                }
                else if end_x > SCREEN_WIDTH as u8{
                    vis_end = cmp::min(SPRITE_WIDTH - (end_x - SCREEN_WIDTH as u8), vis_end);
                }
        
                self.sprite_fetcher.oam_entries[self.sprite_fetcher.oam_entries_len as usize] = 
                    SpriteAttributes::new(end_y, end_x, tile_number, attributes, vis_start, vis_end);
                self.sprite_fetcher.oam_entries_len += 1;
                if self.sprite_fetcher.oam_entries_len == MAX_SPRITES_PER_LINE as u8{
                    break;
                }
            }
        }
        // I beleive using unstable sort here shouldnt be a problem since the above make sure that each sprite is treated well 
        // and knows which part of it should be rendered.
        // I might be wrong so leaving this comment here
        self.sprite_fetcher.oam_entries[0..self.sprite_fetcher.oam_entries_len as usize]
            .sort_unstable_by(|s1:&SpriteAttributes, s2:&SpriteAttributes| s1.x.cmp(&s2.x));
    }

    fn try_push_to_lcd(&mut self){
        if self.bg_fetcher.fifo.len() == 0{
            return;
        }
        if !self.scanline_started{
            let screen_x_indicator = if self.bg_fetcher.rendering_window{self.window_pos.x}else{self.bg_pos.x};
            // discard the next pixel in the bg fifo
            // the bg fifo should start with 8 pixels and not push more untill its empty again
            if FIFO_SIZE as usize - self.bg_fetcher.fifo.len() >= screen_x_indicator as usize % FIFO_SIZE as usize{
                self.scanline_started = true;
            }
            else{
                self.bg_fetcher.fifo.remove();
                return;
            }
        }

        let bg_pixel = self.bg_fetcher.fifo.remove();
        let pixel = self.get_pixel_color(bg_pixel);
        self.push_pixel(Color::into(pixel));
        self.pixel_x_pos += 1;
    }

    fn get_pixel_color(&mut self, bg_pixel:BackgroundPixel) -> Color {
        if self.sprite_fetcher.fifo.len() == 0{
            return self.get_bg_pixel(bg_pixel);
        }
        let sprite_pixel = self.sprite_fetcher.fifo.remove();
        if sprite_pixel.color_index == 0{
            return self.get_bg_pixel(bg_pixel);
        }
        let pixel_oam_attribute = &self.sprite_fetcher.oam_entries[sprite_pixel.oam_entry as usize];
        if self.mode == Mode::CGB{
            // Based on MagenTests ColorBgOamPriority - https://github.com/alloncm/MagenTests
            // in case BG pixel is 0 or BG layer is diabled or both BG and OAM attributes has BG priority disabled
            // draw the OAM pixel else draw the BG pixel
            return if bg_pixel.color_index == 0 || self.lcd_control & BIT_0_MASK == 0 || (!pixel_oam_attribute.attributes.bg_priority && !bg_pixel.attributes.attribute.bg_priority){
                Self::get_color_from_color_ram(&self.obj_color_ram, pixel_oam_attribute.gbc_palette_number, sprite_pixel.color_index)
            }
            else{
                Self::get_color_from_color_ram(&self.bg_color_ram, bg_pixel.attributes.cgb_pallete_number, bg_pixel.color_index)                    
            };
        }
        else{
            if pixel_oam_attribute.attributes.bg_priority && bg_pixel.color_index != 0{
                return self.bg_color_mapping[bg_pixel.color_index as usize];
            }
            else{
                let sprite_pixel = if pixel_oam_attribute.gb_palette_number{
                    self.obj_color_mapping1[sprite_pixel.color_index as usize]
                }
                else{
                    self.obj_color_mapping0[sprite_pixel.color_index as usize]
                };
                return sprite_pixel.expect("Corruption in the object color pallete");
            }
        }
    }

    fn get_bg_pixel(&self, bg_pixel:BackgroundPixel) -> Color {
        return if self.mode == Mode::CGB{
            Self::get_color_from_color_ram(&self.bg_color_ram, bg_pixel.attributes.cgb_pallete_number, bg_pixel.color_index)
        }            
        else{
            self.bg_color_mapping[bg_pixel.color_index as usize]
        };
    }

    fn push_pixel(&mut self, pixel: Pixel) {
        self.screen_buffers[self.current_screen_buffer_index][self.screen_buffer_index] = pixel;
        self.screen_buffer_index += 1;
        if self.screen_buffer_index == SCREEN_WIDTH * SCREEN_HEIGHT{
           self.swap_buffer();
        }
    }

    fn get_color_from_color_ram(color_ram:&[u8;64], pallete: u8, pixel: u8) -> Color {
        const COLOR_PALLETE_SIZE:u8 = 8;
        let pixel_color_index = (pallete * COLOR_PALLETE_SIZE) + (pixel * 2);
        let mut color:u16 = color_ram[pixel_color_index as usize] as u16;
        color |= (color_ram[pixel_color_index as usize + 1] as u16) << 8;
        
        return Color::from(color);
    }
}