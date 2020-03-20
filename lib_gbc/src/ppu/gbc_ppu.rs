use crate::utils::vec2::Vec2;
use crate::machine::vram::VRam;
use crate::machine::memory::Memory;

//const SCREEN_HEIGHT: usize = 144;
//const SCREEN_WIDTH: usize = 160;
const SCREEN_BUFFER_SIZE: usize = 0xFFFF;
const VRAM_START_ADDRESS:u16 = 0x8000;
const VRAM_END_ADDRESS:u16 = 0x97FF;
//const SPRITE_NORMAL_SIZE:u8 = 8;

struct Sprite{
    pixels:[u8;64]
}

pub struct GbcPpu{
    pub screen_buffer:[u8;SCREEN_BUFFER_SIZE],
    pub screen_enable:bool,
    pub window_enable:bool,
    pub sprite_extended:bool,
    pub background_enabled:bool,
    pub gbc_mode:bool,
    pub sprite_enable:bool,
    pub window_tile_map_address:bool,
    pub window_tile_background_map_data_address:bool,
    pub background_tile_map_address:bool,
    pub background_scroll:Vec2<u8>
}

impl Default for GbcPpu{
    fn default()->GbcPpu{
        GbcPpu{
            background_enabled:false,
            background_scroll:Vec2::<u8>{x:0,y:0},
            background_tile_map_address:false,
            gbc_mode:false,
            screen_buffer:[0;SCREEN_BUFFER_SIZE],
            screen_enable:false,
            sprite_enable:false,
            sprite_extended:false,
            window_enable:false,
            window_tile_background_map_data_address:false,
            window_tile_map_address:false
        }
    }
}

impl GbcPpu{
    fn fill_sprite(&self, vram:[u8;16])->Sprite{
        let mut sprite:Sprite = Sprite{pixels:[0;64]};
        for i in (0..16).step_by(2){
            let first_byte:u8 = vram[i];
            let second_byte:u8 = vram[i+1];
            for j in 0..8{
                let mask:u8 = 1<<(7-j);
                sprite.pixels[i+j] = (first_byte & mask)>>7-j;
                sprite.pixels[i+j] = ((second_byte & mask)>>7-j)<<1;
            }
        }

        return sprite;
    }

    fn get_sprites(&self, vram:&VRam)->Vec<Sprite>{
        //let sprite_size = SPRITE_NORMAL_SIZE + (SPRITE_NORMAL_SIZE * self.sprite_extended as u8);
        let mut sprites:Vec<Sprite> = Vec::with_capacity(256);
        let mut vram = vram.clone();
        vram.set_bank(0);
        self.fill_sprites(&mut sprites, &vram);
        if self.gbc_mode{
            vram.set_bank(1);
            self.fill_sprites(&mut sprites, &vram);
        }

        return sprites;
    }

    fn fill_sprites(&self, sprites:&mut Vec<Sprite>, vram:&VRam){
        for i in (VRAM_START_ADDRESS..=VRAM_END_ADDRESS).step_by(16){
            let mut array:[u8;16] = [0;16];
            for j in 0u16..16{
                array[j as usize] = vram.read_current_bank(i+j);
            }

            let sprite: Sprite = self.fill_sprite(array);
            sprites.push(sprite);
        }
    }

    pub fn get_screen_buffer(&mut self,memory :&dyn Memory, vram:&VRam)->Vec<u8>{
        let sprites:Vec<Sprite> = self.get_sprites(vram);
        let start:u16 = if self.background_tile_map_address {0x9800} else{0x9C00};
        let mut screen_buffer:Vec<u8> = Vec::new();
        for i in start..start+0x400{
            let index:u8 = memory.read(i);
            let sprite:&Sprite = &sprites[index as usize];
            screen_buffer.extend_from_slice(&sprite.pixels);
        }

        return screen_buffer;
    }

}