use super::create_default_array;

pub struct FixedSizeQueue<T:Default + Copy, const SIZE:usize>{
    data: [T;SIZE],
    length: usize,
}

impl<T:Default + Copy, const SIZE:usize> FixedSizeQueue<T, SIZE>{
    pub fn new()->Self{
        Self{
            data:create_default_array(),
            length:0,
        }
    }

    pub fn push(&mut self, t:T){
        if self.length < SIZE{
            self.data[self.length] = t;
            self.length += 1;
        }
        else{
            std::panic!("queue is already full, size: {}", SIZE);
        }
    }

    pub fn remove(&mut self)->T{
        if self.length > 0{
            let t = self.data[0];
            for i in 1..self.length{
                self.data[i - 1] = self.data[i];
            }
            self.length -= 1;
            return t;
        }
        
        std::panic!("The fifo is empty");
    }

    pub fn clear(&mut self){
        self.length = 0;
    }

    pub fn len(&self)->usize{
        self.length
    }
}

impl<T:Default + Copy, const SIZE:usize> std::ops::Index<usize> for FixedSizeQueue<T, SIZE>{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.length{
            return &self.data[index];
        }

        std::panic!("Index is out of range");
    }
}

impl<T:Default + Copy, const SIZE:usize> std::ops::IndexMut<usize> for FixedSizeQueue<T, SIZE>{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.length{
            return &mut self.data[index];
        }

        std::panic!("Index is out of range");
    }
}