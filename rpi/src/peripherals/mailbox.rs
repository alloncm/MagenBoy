// This module is wrriten using the following sources
// Linux kernel for the rpi version 4.9 as a referrence - https://github.com/raspberrypi/linux/blob/2aae4cc63e30f99dd152fc63cc4a67ca29e4647b/drivers/firmware/raspberrypi.c
// This tutorial - https://www.rpi4os.com/part5-framebuffer/
// The official mailbox docs - https://github.com/raspberrypi/firmware/wiki/Mailboxes

//Turns out this peripheral needs a non cached memory

use core::mem::size_of;

use super::utils::compile_time_size_assert;

#[cfg(feature = "os")]
pub use std_impl::Mailbox;
#[cfg(not(feature = "os"))]
pub use no_std_impl::Mailbox;

#[repr(C, align(4))]
struct PropertyTagHeader{
    tag:u32,
    buffer_size:u32,

    // On submit, the length of the request (though it doesn't appear to be currently used by the firmware).  
    // On return, the length of the response (always 4 byte aligned), with the low bit set.
    request_response_size:u32   
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq)]
enum PropertyStatus{
    Request = 0,
    Success = 0x8000_0000,
    _Error   = 0x8000_0001
}
compile_time_size_assert!(PropertyStatus, 4);

// The 28 first bits are the address but the last 4 bits represnt the mailbox channel
// by aligning to 16 the last 4 bit are always 0
#[repr(C, align(16))]
struct Message<const DATA_LEN:usize>{
    size:u32,
    status:PropertyStatus,
    tag_header:PropertyTagHeader,
    data:[u32;DATA_LEN],
    property_end_marker:u32
}

impl<const DATA_LEN:usize> Message<DATA_LEN>{
    const MBOX_PROPERTY_END:u32 = 0;

    fn new(tag:u32, data:[u32;DATA_LEN])->Self{
        Self{
            size: size_of::<Self>() as u32,
            status: PropertyStatus::Request,
            tag_header: PropertyTagHeader {
                tag,
                buffer_size: (size_of::<u32>() * DATA_LEN) as u32,
                request_response_size: 0    // zero since unused by the firmare
            },
            data,
            property_end_marker: Self::MBOX_PROPERTY_END
        }
    }
}

#[cfg(not(feature = "os"))]
mod no_std_impl{
    use core::mem::size_of;
    use crate::peripherals::utils::{MmioReg32, compile_time_size_assert, memory_barrier, get_static_peripheral};
    use super::*;

    const MBOX_BASE_OFFSET:usize = 0xB880;
    const MBOX_CHANNEL:u32 = 8;     // free channel for communication from the cpu to the core
    
    #[repr(C, align(4))]
    struct MailboxRegisters{
        read_reg:MmioReg32,
        _res:[u32;5],
        status:MmioReg32,
        _res1:u32,
        write_reg:MmioReg32,
    }
    compile_time_size_assert!(MailboxRegisters, 0x24);

    pub struct Mailbox{
        registers:&'static mut MailboxRegisters,
        uncached_buffer:&'static mut [u8]
    }

    impl Mailbox{
        const STATUS_EMPTY:u32  = 0x4000_0000;
        const STATUS_FULL:u32   = 0x8000_0000;

        pub(in crate::peripherals) fn new()->Mailbox{
            const MAILBOX_BUFFER_SIZE:usize = 0x100;
            #[repr(align(16))]
            struct MailboxBuffer([u8;MAILBOX_BUFFER_SIZE]);
            #[no_mangle]
            #[link_section = ".uncached"]
            static mut MAILBOX_UNCACHED_BUFFER:MailboxBuffer = MailboxBuffer([0;MAILBOX_BUFFER_SIZE]);

            let registers:&mut MailboxRegisters = get_static_peripheral(MBOX_BASE_OFFSET);
            let uncached_buffer:&'static mut [u8] = unsafe{&mut MAILBOX_UNCACHED_BUFFER.0};
            return Mailbox { registers, uncached_buffer };
        }

        pub fn call<const DATA_LEN:usize>(&mut self, tag:u32, data:[u32;DATA_LEN])->[u32;DATA_LEN]{
            if size_of::<Message<DATA_LEN>>()> self.uncached_buffer.len() {
                core::panic!("Error, Message with data len of {} bytes is too large ({}) and cant fit a {} bytes buffer", 
                    DATA_LEN, size_of::<Message<DATA_LEN>>(), self.uncached_buffer.len());
            }

            let uncached_message = unsafe{
                let message = Message::new(tag, data);
                core::ptr::copy_nonoverlapping(&message as *const Message<DATA_LEN> as *const u8, self.uncached_buffer.as_mut_ptr(), size_of::<Message<DATA_LEN>>());
                &*(self.uncached_buffer.as_ptr() as *const Message<DATA_LEN>)
            };
            let mut message_address = (self.uncached_buffer.as_ptr()) as *const Message<DATA_LEN> as u32;
            if message_address & 0xF != 0{
                core::panic!("Error! mbox message is not alligned for 16 bytes")
            }
            message_address += MBOX_CHANNEL;

            memory_barrier();
            while self.registers.status.read() & Self::STATUS_FULL != 0{}   // blocks untill mbox is avaliable
            self.registers.write_reg.write(message_address);

            loop{
                while self.registers.status.read() & Self::STATUS_EMPTY != 0{}    // block untill there is a response (non empty mailbox)
                if self.registers.read_reg.read() == message_address{
                    memory_barrier();
                    if uncached_message.status == PropertyStatus::Success{
                        return uncached_message.data;
                    }
                    core::panic!("Error in mbox call! tag: {:#X}, req_data: {:?}, status: {:#X}, res_data: {:?}", 
                        tag, data, uncached_message.status as u32, uncached_message.data);
                }
            }
        }
    }
}

#[cfg(feature = "os")]
mod std_impl{
    use libc::{c_int, c_void};

    use crate::peripherals::utils::libc_abort;

    use super::*;

    pub struct Mailbox{
        mbox_fd: c_int,
    }

    impl Mailbox{
        const MAILBOX_IOCTL_PROPERTY:libc::c_ulong = nix::request_code_readwrite!(100, 0, std::mem::size_of::<*mut libc::c_void>());

        pub(in crate::peripherals) fn new()->Self{
            let fd = unsafe{libc::open(std::ffi::CStr::from_bytes_with_nul(b"/dev/vcio\0").unwrap().as_ptr(), 0)};
            if fd < 0{
                std::panic!("Error while opening vc mailbox");
            }

            Self { mbox_fd: fd }
        }

        pub fn call<const SIZE:usize>(&mut self, tag:u32, data:[u32;SIZE])->[u32;SIZE]{
            let mut message = Message::<SIZE>::new(tag, data);
            return self.send_message(&mut message);
        }

        fn send_message<const SIZE:usize>(&self, message:&mut Message<SIZE>)->[u32;SIZE]{
            let raw_message = message as *mut Message<SIZE> as *mut c_void;
            let ret = unsafe{
                // Using libc::ioctl and not nix high level abstraction over it cause Im sending a *void and not more 
                // concrete type and the nix macro will mess the types for us. I belive it could work with nix after some modification 
                // of the way Im handling this but Im leaving this as it for now. sorry!
                libc::ioctl(self.mbox_fd, Self::MAILBOX_IOCTL_PROPERTY, raw_message)
            };
            if ret < 0{
                libc_abort("Error in ioctl call");
            }
            if message.status != PropertyStatus::Success{
                std::panic!("Error in mbox call! tag: {:#X}, data: {:?}, status: {:#X}",
                    message.tag_header.tag, message.data, message.status as u32);
            }

            // The return value of the command is located at the first int in the data section (for more info see the Mailbox docs)
            return message.data;
        }
    }

    impl Drop for Mailbox{
        fn drop(&mut self) {
            unsafe{
                let result = libc::close(self.mbox_fd);
                if result != 0{
                    libc_abort("Error while closing the mbox fd");
                }
            }
        }
    }
}