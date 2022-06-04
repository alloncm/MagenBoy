pub mod ili9341_controller;
pub mod gpio_joypad_provider;

cfg_if::cfg_if!{if #[cfg(feature = "raw-spi")]{
    mod dma;
    mod raw_spi;
    pub type SpiType = raw_spi::RawSpi;
}else{
#[cfg(not(feature = "raw-spi"))]
    mod spi;
    pub type SpiType = spi::RppalSpi;
}}


fn libc_abort(message:&str){
    std::io::Result::<&str>::Err(std::io::Error::last_os_error()).expect(message);
}

macro_rules! decl_write_volatile_field{
    ($function_name:ident, $field_name:ident) =>{
        #[inline] unsafe fn $function_name(&mut self,value:u32){
            std::ptr::write_volatile(&mut self.$field_name , value);
        }
    }
}

macro_rules! decl_read_volatile_field{
    ($function_name:ident, $field_name:ident) =>{
        #[inline] unsafe fn $function_name(&mut self)->u32{
            std::ptr::read_volatile(&self.$field_name)
        }
    }
}

pub(self) use {decl_read_volatile_field, decl_write_volatile_field};