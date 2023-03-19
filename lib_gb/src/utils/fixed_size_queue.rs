pub struct FixedSizeQueue<T:Copy, const SIZE:usize>{
    // According to the docs Vec should not be moved in memory if it not modified
    // Im modifing it but not increasing its allocated size once its allocated so I hope this will work for me
    // and I wont get weird memory issues
    _data: Vec<T>,   // This field is not use directly only through pointers aquired in the new() function
    end_alloc_pointer: *const T,
    start_alloc_pointer: *const T,
    data_pointer: *mut T,
    base_data_pointer: *mut T,
    length: usize,
}

impl<T:Copy + Default, const SIZE:usize> FixedSizeQueue<T, SIZE>{
    pub fn new()->Self{
        let data = vec![T::default();SIZE];
        let mut s = Self{
            _data: data,
            length:0,
            base_data_pointer: std::ptr::null_mut(),
            data_pointer: std::ptr::null_mut(),
            end_alloc_pointer: std::ptr::null_mut(),
            start_alloc_pointer: std::ptr::null_mut(),
        };

        s.base_data_pointer = s._data.as_mut_ptr();
        s.data_pointer = s._data.as_mut_ptr();
        s.start_alloc_pointer = s._data.as_ptr();
        unsafe{s.end_alloc_pointer = s._data.as_ptr().add(SIZE)};

        return s;
    }

    pub fn push(&mut self, t:T){
        if self.length < SIZE{
            unsafe{
                if self.data_pointer == self.end_alloc_pointer as *mut T{
                    self.data_pointer = self.start_alloc_pointer as *mut T;
                }
                *self.data_pointer = t;
                self.data_pointer = self.data_pointer.add(1);
            }
            self.length += 1;
        }
        else{
            std::panic!("queue is already full, size: {}", SIZE);
        }
    }

    pub fn remove(&mut self)->T{
        if self.length > 0{
            unsafe{
                let t = *self.base_data_pointer;
                self.base_data_pointer = self.base_data_pointer.add(1);
                if self.base_data_pointer == self.end_alloc_pointer as *mut T{
                    self.base_data_pointer = self.start_alloc_pointer as *mut T;
                }

                self.length -= 1;
                return t;
            }
        }
        
        std::panic!("The fifo is empty");
    }

    pub fn clear(&mut self){
        self.length = 0;
        self.data_pointer = self.start_alloc_pointer as *mut T;
        self.base_data_pointer = self.start_alloc_pointer as *mut T;
    }

    pub fn len(&self)->usize{
        self.length
    }

    pub fn fill(&mut self, value:&[T;SIZE]){
        unsafe{
            self.base_data_pointer = self.start_alloc_pointer as *mut T;
            std::ptr::copy_nonoverlapping(value.as_ptr(), self.base_data_pointer, SIZE);
            self.length = SIZE;
            self.data_pointer = self.end_alloc_pointer as *mut T;
        }
    }
}

impl<T:Copy, const SIZE:usize> FixedSizeQueue<T, SIZE>{
    #[inline] 
    fn get_index_ptr(&self, index:usize)->*const T{
        if index < self.length{
            unsafe{
                if self.base_data_pointer.add(index) >= self.end_alloc_pointer as *mut T{
                    let wrap_offset = self.end_alloc_pointer.offset_from(self.base_data_pointer) as usize;
                    return self.start_alloc_pointer.add(index - wrap_offset);
                }
                else{
                    return self.base_data_pointer.add(index);
                }
            }
        }
        std::panic!("Index is out of range");
    }
}

impl<T:Copy, const SIZE:usize> std::ops::Index<usize> for FixedSizeQueue<T, SIZE>{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe{&*(self.get_index_ptr(index))}
    }
}

impl<T:Copy, const SIZE:usize> std::ops::IndexMut<usize> for FixedSizeQueue<T, SIZE>{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe{&mut *(self.get_index_ptr(index) as *mut T)}
    }
}