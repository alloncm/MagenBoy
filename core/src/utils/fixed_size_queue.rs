use core::{ops::{IndexMut, Index}, ptr};

use super::global_static_alloctor::static_alloc_ptr;

pub struct FixedSizeQueue<T, const SIZE:usize>{
    end_alloc_pointer: *mut T,
    start_alloc_pointer: *mut T,
    data_pointer: *mut T,
    base_data_pointer: *mut T,
    length: usize,
}

impl<T:Copy + Default, const SIZE:usize> FixedSizeQueue<T, SIZE>{
    pub fn new()->Self{
        let data_ptr = static_alloc_ptr(SIZE).as_ptr();
        let mut s = Self{
            length:0,
            base_data_pointer: ptr::null_mut(),
            data_pointer: ptr::null_mut(),
            end_alloc_pointer: ptr::null_mut(),
            start_alloc_pointer: ptr::null_mut(),
        };

        s.base_data_pointer = data_ptr;
        s.data_pointer = data_ptr;
        s.start_alloc_pointer = data_ptr;

        // SAFETY: Adding the len of the buffer to its base address resulting in off by 1 ptr which is safe
        unsafe{s.end_alloc_pointer = data_ptr.add(SIZE)};

        return s;
    }

    pub fn push(&mut self, t:T){
        if self.length < SIZE{
            unsafe{
                if self.data_pointer == self.end_alloc_pointer {
                    self.data_pointer = self.start_alloc_pointer;
                }
                *self.data_pointer = t;
                self.data_pointer = self.data_pointer.add(1);
            }
            self.length += 1;
        }
        else{
            core::panic!("queue is already full, size: {}", SIZE);
        }
    }

    pub fn remove(&mut self)->T{
        if self.length > 0{
            unsafe{
                let t = *self.base_data_pointer;
                self.base_data_pointer = self.base_data_pointer.add(1);
                if self.base_data_pointer == self.end_alloc_pointer {
                    self.base_data_pointer = self.start_alloc_pointer;
                }

                self.length -= 1;
                return t;
            }
        }
        
        core::panic!("The fifo is empty");
    }

    pub fn clear(&mut self){
        self.length = 0;
        self.data_pointer = self.start_alloc_pointer;
        self.base_data_pointer = self.start_alloc_pointer;
    }

    pub fn len(&self)->usize{
        self.length
    }

    pub fn fill(&mut self, value:&[T;SIZE]){
        unsafe{
            self.base_data_pointer = self.start_alloc_pointer;
            ptr::copy_nonoverlapping(value.as_ptr(), self.base_data_pointer, SIZE);
            self.length = SIZE;
            self.data_pointer = self.end_alloc_pointer;
        }
    }
}

impl<T:Copy, const SIZE:usize> FixedSizeQueue<T, SIZE>{
    #[inline] 
    fn get_index_ptr(&self, index:usize)->*mut T{
        if index < self.length{
            unsafe{
                if self.base_data_pointer.add(index) >= self.end_alloc_pointer{
                    let wrap_offset = self.end_alloc_pointer.offset_from(self.base_data_pointer) as usize;
                    return self.start_alloc_pointer.add(index - wrap_offset);
                }
                else{
                    return self.base_data_pointer.add(index);
                }
            }
        }
        core::panic!("Index is out of range");
    }
}

impl<T:Copy, const SIZE:usize> Index<usize> for FixedSizeQueue<T, SIZE>{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe{&*(self.get_index_ptr(index))}
    }
}

impl<T:Copy, const SIZE:usize> IndexMut<usize> for FixedSizeQueue<T, SIZE>{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe{&mut *(self.get_index_ptr(index))}
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    
    #[test]
    fn test_fifo_ub(){
        let mut fifo = FixedSizeQueue::<u8, 8>::new();
        fifo.push(10);
    }

    #[test]
    fn test_fifo(){
        let mut fifo = FixedSizeQueue::<u8, 8>::new();
        fifo.push(10);
        fifo.push(22);

        assert_eq!(fifo.len(), 2);
        assert_eq!(fifo[0], 10);
        assert_eq!(fifo[1], 22);

        fifo.remove();
        assert_eq!(fifo.len(), 1);
        assert_eq!(fifo[0], 22);

        fifo[0] = 21;
        assert_eq!(fifo[0], 21);
    }

    #[test]
    fn test_fifo_wrapping_around(){
        let mut fifo = FixedSizeQueue::<u8, 3>::new();

        check_push_and_remove(&mut fifo);
        check_push_and_remove(&mut fifo);
        check_push_and_remove(&mut fifo);

        fifo.push(10);
        fifo.push(22);
        fifo.remove();
        assert_eq!(fifo.len(), 1);
        assert_eq!(fifo[0], 22);

        fifo[0] = 21;
        assert_eq!(fifo[0], 21);
    }

    fn check_push_and_remove(fifo: &mut FixedSizeQueue<u8, 3>) {
        fifo.push(10);
        fifo.push(22);
        fifo.push(33);

        assert_eq!(fifo.len(), 3);
        assert_eq!(fifo[0], 10);
        assert_eq!(fifo[1], 22);
        assert_eq!(fifo[2], 33);

        assert_eq!(fifo.remove(), 10);
        assert_eq!(fifo.remove(), 22);
        assert_eq!(fifo.remove(), 33);
        assert_eq!(fifo.len(), 0);
    }

    #[test]
    #[should_panic]
    fn panic_on_fifo_full(){
        let mut fifo = FixedSizeQueue::<u8, 3>::new();
        fifo.push(1);
        fifo.push(2);
        fifo.push(3);

        //should panic
        fifo.push(1);
    }

    #[test]
    #[should_panic]
    fn panic_on_get_fifo_index_out_of_range(){
        let mut fifo = FixedSizeQueue::<u8, 3>::new();
        fifo.push(1);
        fifo.push(2);

        //should panic
        let _ = fifo[2];
    }

    #[test]
    #[should_panic]
    fn panic_on_fifo_set_index_out_of_range(){
        let mut fifo = FixedSizeQueue::<u8, 3>::new();
        fifo.push(1);
        fifo.push(2);

        //should panic
        fifo[2] = 4;
    }

    #[test]
    fn fill_fills_the_fifo(){
        let mut fifo = FixedSizeQueue::<u8, 8>::new();
        fifo.push(1);
        fifo.push(1);
        fifo.push(1);

        fifo.remove();
        fifo.remove();
        
        fifo.fill(&[0;8]);

        assert_eq!(fifo.len(), 8);
        for i in 0..8{
            assert_eq!(fifo[i], 0);
        }
    }

    #[test]
    fn fifo_index_check_happyflow(){
        let mut fifo = FixedSizeQueue::<u8, 8>::new();
        for i in 0..8{
            fifo.push(i);
        }
        for _ in 0..6{
            fifo.remove();
        }
        for i in 0..6{
            fifo.push(i);
        }

        assert_eq!(fifo[0], 6);
        assert_eq!(fifo[1], 7);
        assert_eq!(fifo[2], 0);
        assert_eq!(fifo[3], 1);
        assert_eq!(fifo[4], 2);
        assert_eq!(fifo[5], 3);
    }
}