pub const BIT_0_MASK:u8 = 1;
pub const BIT_1_MASK:u8 = 1 << 1;
pub const BIT_2_MASK:u8 = 1 << 2;
pub const BIT_3_MASK:u8 = 1 << 3;
pub const BIT_4_MASK:u8 = 1 << 4;
pub const BIT_5_MASK:u8 = 1 << 5;
pub const BIT_6_MASK:u8 = 1 << 6;
pub const BIT_7_MASK:u8 = 1 << 7;

pub const BIT_9_MASK:u16 = 1 << 9;

#[inline]
pub fn flip_bit_u8(value:&mut u8, bit_number:u8, set:bool){
    let mask = !(1 << bit_number);
    let bit_value = (set as u8) << bit_number;
    *value |= bit_value;            // setting the bit if set or do nothing if unset
    *value &= mask | bit_value;     // masking value with the mask and the bit (if set)
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


#[cfg(test)]
mod tests{
    use super::flip_bit_u8;

    #[test]
    fn test_flip_bit_u8_set(){
        let mut values = [0, 0x01, 0x02, 0x03, 0x14];
        let expected = [0x10, 0x11, 0x12, 0x13, 0x14];

        for v in &mut values{
            flip_bit_u8(v, 4, true);
        }

        assert_eq!(values, expected);
    }

    #[test]
    fn test_flip_bit_u8_unset(){
        let mut values = [0x10, 0x11, 0x12, 0x13, 0x04];
        let expected = [0, 0x01, 0x02, 0x03, 0x04];

        for v in &mut values{
            flip_bit_u8(v, 4, false);
        }

        assert_eq!(values, expected);
    }
}