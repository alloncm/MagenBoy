use core::mem::MaybeUninit;

pub mod vec2;
pub mod memory_registers;
pub mod bit_masks;
pub mod fixed_size_queue;
pub mod static_allocator;
pub mod global_static_alloctor;
cfg_if::cfg_if!{ if #[cfg(feature = "dbg")]{
    pub mod stack_string;

    mod fixed_size_set;
    pub use fixed_size_set::FixedSizeSet;
}}

// Frequency in m_cycles (m_cycle = 4 t_cycles)
pub const GB_FREQUENCY:u32 = 4_194_304 / 4;

//CPU frequrncy: 4,194,304 / 59.727~ / 4 == 70224 / 4
pub const CYCLES_PER_FRAME:u32 = 17556;

pub fn create_default_array<T:Default,const SIZE:usize>()->[T;SIZE]{
    create_array(||T::default())
}

pub fn create_array<T, F:FnMut()->T,const SIZE:usize>(mut func:F)->[T;SIZE]{
    let mut data: [MaybeUninit<T>; SIZE] = unsafe{MaybeUninit::uninit().assume_init()};

    for elem in &mut data[..]{
        *elem = MaybeUninit::new(func());
    }
    unsafe{
        let casted_data = core::ptr::read(&data as *const [MaybeUninit<T>;SIZE] as *const [T;SIZE]);
        core::mem::forget(data);
        return casted_data;
    }
}