use core::{sync::atomic::{AtomicBool, Ordering}, cell::UnsafeCell};

pub struct Mutex<T:?Sized> {
    lock:AtomicBool,
    data:UnsafeCell<T>,
}

unsafe impl<T:?Sized + Send> Send for Mutex<T> {}
unsafe impl<T:?Sized + Send> Sync for Mutex<T> {}

impl<TData> Mutex<TData>{
    pub const fn new(data:TData)->Mutex<TData>{
        Mutex { data:UnsafeCell::new(data), lock:AtomicBool::new(false) }
    }
    
    pub fn lock<'a, TReturn>(&'a self, callback: impl FnOnce(&'a mut TData)->TReturn)->TReturn{
        // block untill given access to lock the mutex
        while self.lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok(){}
        let data = unsafe{&mut *self.data.get()};
        let res = callback(data);
        // free the mutex
        self.lock.store(false, Ordering::SeqCst);

        return res;
    }
}