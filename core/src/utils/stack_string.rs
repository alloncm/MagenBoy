use core::{convert::From, fmt::{Write, Arguments}};

#[derive(Clone, Copy)]
pub struct StackString<const SIZE:usize>{
    data:[u8; SIZE],
    size: usize
}

impl<const SIZE:usize> Default for StackString<SIZE>{
    fn default() -> Self {
        Self { data: [0;SIZE], size: 0 }
    }
}

impl<const SIZE:usize> From<&str> for StackString<SIZE>{
    fn from(value: &str) -> Self {
        let bytes = value.as_bytes();
        if bytes.len() > SIZE{
            core::panic!("Data is too large for the string");
        }
        let mut str = Self::default();
        str.append(bytes);
        return str;
    }
}

impl<const SIZE:usize> StackString<SIZE>{
    pub fn from_args(args:Arguments)->Self{
        let mut str = Self::default();
        str.write_fmt(args).unwrap();
        return str;
    }

    pub fn append(&mut self, data_to_append:&[u8]){
        if self.size + data_to_append.len() > SIZE{
            core::panic!("Error!, trying to append to stack string with too much data");
        }
        self.data[self.size .. self.size + data_to_append.len()].copy_from_slice(data_to_append);
        self.size += data_to_append.len();
    }

    pub fn as_str<'a>(&'a self)->&'a str{
        return core::str::from_utf8(&self.data[0..self.size]).unwrap();
    }
}

impl<const SIZE:usize> Write for StackString<SIZE>{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        if self.size + s.len() > SIZE{
            return core::fmt::Result::Err(core::fmt::Error);
        }
        self.append(bytes);
        return core::fmt::Result::Ok(());
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_append(){
        let mut str = StackString::<10>::default();
        str.append(b"hello");
        assert_eq!(str.as_str(), "hello");
    }

    #[test]
    fn test_append_u8(){
        let mut str = StackString::<10>::default();
        str.append(&[0x56]);
        assert_eq!(str.as_str(), "hello");
    }
}