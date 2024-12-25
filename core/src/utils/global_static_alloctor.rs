use core::{mem::size_of, ptr::{slice_from_raw_parts_mut, write_unaligned, NonNull}};

use super::static_allocator::{StaticAllocator, Layout};

const STATIC_MEMORY_SIZE:usize = 0x100_0000;
static mut MEMORY:[u8;STATIC_MEMORY_SIZE] = [0;STATIC_MEMORY_SIZE];

// SAFETY: Only access to the global static var
static mut ALLOCATOR:StaticAllocator = unsafe{StaticAllocator::new(MEMORY.as_mut_ptr(), MEMORY.len())};

pub fn static_alloc<T>(t:T)->&'static mut T{
    let layout = Layout::from_type::<T>();
    unsafe{
        let ptr = ALLOCATOR.alloc(layout).as_ptr() as *mut T;
        write_unaligned(ptr, t);
        return &mut *ptr;
    }
}

pub fn static_alloc_array<T:Default>(len:usize)->&'static mut [T]{
    let layout = Layout::new(len * size_of::<T>());
    unsafe{
        let ptr = ALLOCATOR.alloc(layout).as_ptr() as *mut T;
        let slice:&'static mut[T] = &mut *slice_from_raw_parts_mut(ptr, len);
        for t in slice.iter_mut(){
            write_unaligned(t as *mut T, T::default());
        }

        return slice;
    }
}

pub fn static_alloc_ptr<T>(amount:usize)->NonNull<T>{
    let layout = Layout::with_align(amount * size_of::<T>(), align_of::<T>());
    // SAFETY: Accessing the global allocator, since this is a single thread process it is safe
    return unsafe{ALLOCATOR.alloc(layout).cast()};
}