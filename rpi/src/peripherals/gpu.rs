use crate::peripherals::{PERIPHERALS, Tag};

// using GpuMemory cause I need a memory that is not cached by the cpu caches (L1, L2)
pub(super) struct GpuMemory{
    pub virtual_address_ptr:usize,
    pub bus_address:u32,
    mailbox_memory_handle:u32,
    pub size:u32
}

impl GpuMemory{
    const MEM_ALLOC_FLAG_DIRECT:usize = 1 << 2;
    const MEM_ALLOC_FLAG_COHERENT:usize = 1 << 3;
    const PAGE_SIZE:u32 = 4096;

    // This function converts the from the bus address of the SDRAM uncached memory to the arm physical address
    // Notice that supposed to work only for this type of memory
    const fn bus_to_phys(bus_address:u32)->u32{bus_address & !0xC000_0000}

    // Using the Mailbox interface to allocate memory on the gpu
    pub(super) fn allocate(size:u32)->GpuMemory{
        let mbox = unsafe{PERIPHERALS.get_mailbox()};
        let flags = (Self::MEM_ALLOC_FLAG_COHERENT | Self::MEM_ALLOC_FLAG_DIRECT) as u32;

        log::debug!("Trying to allocate: {} memory", size);
        // Result for alloc memory call is in the first u32 of the buffer
        let handle = mbox.call(Tag::AllocateMemory, [size, Self::PAGE_SIZE, flags])[0];
        // This is not documented well but after testing - on out of Gpu memory mailbox returns handle = 0
        if handle == 0{
            core::panic!("Error allocating Gpu memory! perhaps there is not enough free Gpu memory");
        }

        // The result for lock memory call is in the first u32 of the buffer
        let bus_address = mbox.call(Tag::LockMemory, [handle])[0];
        // This is not documented well but after testing - on invalid handle mailbox returns bus_address = 0
        if bus_address == 0{
            core::panic!("Error locking Gpu memory!");
        }

        let address = Self::bus_to_phys(bus_address);
        #[cfg(feature = "os")]
        let address = unsafe{libc::mmap(
            std::ptr::null_mut(),
            size as libc::size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            crate::peripherals::bcm_host::BcmHost::get().get_mem_fd(),
            address as libc::off_t
        )};

        return GpuMemory { virtual_address_ptr: address as usize, bus_address, mailbox_memory_handle:handle, size }
    }

    fn release(&self){
        #[cfg(feature = "os")]
        unsafe{
            if libc::munmap(self.virtual_address_ptr as *mut libc::c_void, self.size as libc::size_t) != 0 {
                crate::peripherals::utils::libc_abort("Error while trying to unmap gpu memory");
            }
        }
        let mbox = unsafe{PERIPHERALS.get_mailbox()};
        if mbox.call(Tag::UnlockMemory, [self.mailbox_memory_handle])[0] != 0{
            core::panic!("Error while trying to unlock gpu memory using mailbox");
        }
        if mbox.call(Tag::ReleaseMemory, [self.mailbox_memory_handle])[0] != 0{
            core::panic!("Error while trying to release gpu memory using mailbox");
        }
    }
}

impl Drop for GpuMemory{
    fn drop(&mut self) {
        self.release();
    }
}