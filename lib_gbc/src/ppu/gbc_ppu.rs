use crate::utils::vec2::Vec2;
use crate::machine::memory::Memory;

const SCREEN_HEIGHT: usize = 144;
const SCREEN_WIDTH: usize = 160;
const SCREEN_BUFFER_SIZE: usize = 0xFFFF;
const VRAM_START_ADDRESS:u16 = 0x8000;
const VRAM_END_ADDRESS:u16 = 0x97FF;
const SPRITE_NORMAL_SIZE:u8 = 8;

struct Sprite{
    pixels:[u8;64]
}

pub struct GbcPpu<'a>{
    pub screen_cordinates: Vec2<u8>,
    pub window_cordinates: Vec2<u8>,
    pub screen_buffer:[u8;SCREEN_BUFFER_SIZE],
    pub screen_enable:bool,
    pub windows_enable:bool,
    pub sprite_extended:bool,
    pub background_enabled:bool,
    memory:&'a dyn Memory
}

impl<'a> GbcPpu<'a>{

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

    fn get_screen_buffer(&self)->Vec<Sprite>{
        let sprite_size = SPRITE_NORMAL_SIZE + (SPRITE_NORMAL_SIZE * self.sprite_extended as u8);
        let sprites:Vec<Sprite> = Vec::with_capacity(256);
        for i in (VRAM_START_ADDRESS..=VRAM_END_ADDRESS).step_by(16){
            let mut array:[u8;16] = [0;16];
            for j in 0u16..16{
                array[j as usize] = self.memory.read(i+j);
            }

            self.fill_sprite(array);
        }

        return sprites;
    }
}