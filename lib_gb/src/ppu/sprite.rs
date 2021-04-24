pub trait Sprite{
    fn size(&self)->u8;
    fn flip_x(&mut self);
    fn flip_y(&mut self);
    fn get_pixel(&self, pos:u8)->u8;
    fn set_pixel(&mut self, pos:u8, pixel:u8);
}

pub fn copy_pixels(sprite:&mut dyn Sprite, index:u8, pixels:&[u8]){
    for i in 0..pixels.len(){
        sprite.set_pixel(index * 8 + i as u8, pixels[i]) ;
    }
}