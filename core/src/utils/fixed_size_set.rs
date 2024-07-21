pub struct FixedSizeSet<T, const SIZE:usize>{
    data:[T;SIZE],
    size:usize,
}

impl<T:Default + Copy + PartialEq, const SIZE:usize> FixedSizeSet<T, SIZE>{
    pub fn new()->Self{
        Self{ data: [T::default(); SIZE], size: 0 }
    }

    pub fn add(&mut self, value:T){
        if self.data.contains(&value){
            return;
        }
        self.data[self.size] = value;
        self.size += 1;
    }

    pub fn try_remove(&mut self, value:T)->bool{
        if !self.data.contains(&value) || self.size == 0 {
            return false;
        }
        let mut found = false;
        for i in 0..self.size {
            if found {
                self.data[i-1] = self.data[i];
            }
            if value == self.data[i]{
                found = true;
            }
        }
        self.size -= 1;
        return true;
    }

    pub fn as_slice(&self)->&[T]{&self.data[..self.size]}
}

mod tests{
    use super::*;

    #[test]
    fn test_fixed_size_set_add(){
        let mut set: FixedSizeSet<u32, 10> = FixedSizeSet::new();
        for i in 0..10{
            set.add(i);
            set.add(i);
        }

        assert!(set.data.into_iter().eq((0..10).into_iter()));
    }
    
    #[test]
    fn test_fixed_size_set_try_remove(){
        let mut set: FixedSizeSet<u32, 10> = FixedSizeSet::new();
        set.add(1);
        
        assert_eq!(set.try_remove(2), false);
        assert_eq!(set.try_remove(1), true);
        assert_eq!(set.size, 0);
    }
}