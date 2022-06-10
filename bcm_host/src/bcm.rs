
use libc::{c_uint, c_void};

// linking to - https://github.com/raspberrypi/firmware/blob/master/opt/vc/include/bcm_host.h
extern "C"{
    // There is no need to link to the init and deinit functions. it looks like they are needed only for the gpu dispnmx stuff
    pub fn bcm_host_get_peripheral_address()->c_uint;   // returns the bcm bus address. should be 0xFE00_0000 on RPI4
    pub fn bcm_host_get_peripheral_size()->c_uint;      // returns the bcm bus size. should be 0x180_0000 on RPI4
}

// This struct is here to managed the lifetime of the bcm2835 ptr and the memory fd
pub struct BcmHost{
    ptr:*mut c_void,
    mem_fd: libc::c_int
}

impl BcmHost {
    pub fn new()->Self{
        let mem_fd = unsafe{libc::open(std::ffi::CStr::from_bytes_with_nul(b"/dev/mem\0").unwrap().as_ptr(), libc::O_RDWR | libc::O_SYNC)};
        
        if mem_fd < 0{
            libc_abort("bad file descriptor");
        }

        let bus_peripherals_address = unsafe{bcm_host_get_peripheral_address()};
        let bus_peripherals_size = unsafe{bcm_host_get_peripheral_size()};

        log::info!("BCM host peripherals address: {:#X}, size: {:#X}", bus_peripherals_address, bus_peripherals_size);
        
        let bcm2835 = unsafe{libc::mmap(
            std::ptr::null_mut(), 
            bus_peripherals_size as usize,
            libc::PROT_READ | libc::PROT_WRITE, 
            libc::MAP_SHARED, 
            mem_fd,
            bus_peripherals_address as libc::off_t
        )};

        if bcm2835 == libc::MAP_FAILED{
            libc_abort("FATAL: mapping /dev/mem failed!");
        }

        BcmHost { ptr: bcm2835, mem_fd }
    }

    pub fn get_ptr(&self, offset:usize)->*mut c_void{
        unsafe{self.ptr.add(offset)}
    }

    pub fn get_fd(&self)->libc::c_int{
        self.mem_fd
    }
}

impl Drop for BcmHost{
    fn drop(&mut self) {
        unsafe{
            let bus_peripherals_size = bcm_host_get_peripheral_size();
            let result = libc::munmap(self.ptr, bus_peripherals_size as usize);
            if result != 0{
                libc_abort("Error while unmapping the mmio memory");
            }

            let result = libc::close(self.mem_fd);
            if result != 0{
                libc_abort("Error while closing the mem_fd");
            }
        }
    }
}

fn libc_abort(message:&str){
    std::io::Result::<&str>::Err(std::io::Error::last_os_error()).expect(message);
}