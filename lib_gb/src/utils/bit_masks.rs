pub const BIT_0_MASK:u8 = 1;
pub const BIT_1_MASK:u8 = 1 << 1;
pub const BIT_2_MASK:u8 = 1 << 2;
pub const BIT_3_MASK:u8 = 1 << 3;
pub const BIT_4_MASK:u8 = 1 << 4;
pub const BIT_5_MASK:u8 = 1 << 5;
pub const BIT_6_MASK:u8 = 1 << 6;
pub const BIT_7_MASK:u8 = 1 << 7;

pub const BIT_9_MASK:u16 = 1 << 9;

pub fn flip_bit_u8(value:&mut u8, bit_number:u8, set:bool){
    let mask = 1 << bit_number;
    if set{
        *value |= mask;
    }
    else{
        let inverse_mask = !mask;
        *value &= inverse_mask;
    }
}

pub fn flip_bit_u16(value:&mut u16, bit_number:u8, set:bool){
    let mask = 1 << bit_number;
    if set{
        *value |= mask;
    }
    else{
        let inverse_mask = !mask;
        *value &= inverse_mask;
    }
}
