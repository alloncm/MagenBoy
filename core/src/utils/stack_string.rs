use core::{convert::From, fmt::Write};

#[derive(Clone, Copy)]
pub struct StackString<const MAX_SIZE:usize>{
    data:[u8; MAX_SIZE],
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

impl<const SIZE:usize> AsRef<str> for StackString<SIZE>{
    fn as_ref(&self) -> &str {self.as_str()}
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
        str.append(&[b'h']);
        assert_eq!(str.as_str(), "h");
    }

    #[test]
    fn test_from(){
        let str = "hello world";

        let _: StackString<11> = StackString::from(str);
        let _: StackString<12> = StackString::from(str);
        let _: StackString<20> = StackString::from(str);
    }

    #[test]
    #[should_panic]
    fn test_from_panic(){
        let str = "hello world";

        let _: StackString<10> = StackString::from(str);
    }

    #[test]
    fn test_as_str(){
        let data = b"hello fucker";
        let ss = StackString{ data:data.clone(), size:  data.len()};

        assert_eq!(ss.as_str(), "hello fucker");
    }

    #[test]
    fn test_write_str(){
        let bstr = b"hello";
        let mut data = [0;20];
        data[0..bstr.len()].copy_from_slice(bstr);
        let mut ss: StackString<20> = StackString{ data, size:  bstr.len()};

        ss.write_str(" fucker").unwrap();
        assert_eq!(&ss.data[0..ss.size], b"hello fucker")
    }

    #[test]
    fn test_write_str_error(){
        let bstr = b"hello";
        let mut data = [0;20];
        data[0..bstr.len()].copy_from_slice(bstr);
        let mut ss: StackString<20> = StackString{ data, size:  bstr.len()};

        let res: Result<(), core::fmt::Error> = ss.write_str(" fucker djakdjaslkdjskl");
        assert_eq!(res, Result::Err(core::fmt::Error));
    }
}