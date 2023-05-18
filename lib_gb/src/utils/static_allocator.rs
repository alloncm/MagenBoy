use core::{mem::{size_of, align_of}, ptr::NonNull};

pub struct Layout{
    size:usize, 
    align:usize
}

impl Layout{
    pub fn new(size:usize)->Self{
        let default_align = size_of::<usize>();
        Self::with_align(size, default_align)
    }
    pub fn from_type<T>()->Self{
        Self::with_align(size_of::<T>(), align_of::<T>())
    }
    pub fn with_align(size:usize, align:usize)->Self{
        if !Self::is_2_power(align) {
            core::panic!("Layout alignment must be a power of 2 but was: {}", align);
        }
        Self { size, align }
    }

    fn is_2_power(x:usize)->bool{
        (x & (x - 1)) == 0
    }
}

pub struct StaticAllocator{
    buffer:&'static [u8],
    allocation_size:usize
}

impl StaticAllocator{
    pub const fn new(buffer:&'static [u8])->Self{
        Self{ buffer, allocation_size: 0 }
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> NonNull<u8> {
        let allocation_address = self.buffer.as_ptr().add(self.allocation_size) as usize;
        let aligned_address = Self::align_address(allocation_address, layout.align);
        self.allocation_size += layout.size + (aligned_address - allocation_address);

        if self.allocation_size > self.buffer.len(){
            core::panic!("Allocation failed, allocator is out of static memory, pool size: {}, allocation req: {}", self.buffer.len(), layout.size);
        }
        return NonNull::new_unchecked(aligned_address as *mut u8);
    }

    fn align_address(address:usize, alignment:usize)->usize{
        let reminder = address % alignment;
        return if reminder != 0 {address - reminder + alignment} else {address};
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_alloc_alignment(){
        unsafe{
            static mut BUFFER:[u8;100] = [0;100];
            let mut allocator = StaticAllocator::new(&mut BUFFER);
            let aligns = 5;
            for a in 1..aligns{
                let align = 1 << a;
                let ptr = allocator.alloc(Layout { size: 1, align });
                // verify the address is aligned
                assert_eq!(ptr.as_ptr() as usize & (align - 1), 0);
            }
        }
    }

    #[test]
    fn test_create_layout(){
        struct TestType{
            _x:u32
        }
        let _ = Layout::from_type::<TestType>();
    }

    #[test]
    #[should_panic]
    fn test_unaligned_layout(){
        let _ = Layout::with_align(1, 3);
    }
}