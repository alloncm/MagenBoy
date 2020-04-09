use std::vec::Vec;

pub struct Surface{
    pub pixels_data:Vec<u32>,
    pub width:u32,
    pub height:u32
}

impl Surface{
    pub fn new(data:Vec<u32>, width:u32, height:u32)->Self{
        if data.len() != (width * height) as usize{
            std::panic!("invalid surface data dimensions do not match");
        }

        return Surface{
            pixels_data:data,
            width:width,
            height:height
        }
    }
}