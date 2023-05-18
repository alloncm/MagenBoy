// This module is wrriten using the following sources
// Linux kernel for the rpi version 4.9 as a referrence - https://github.com/raspberrypi/linux/blob/2aae4cc63e30f99dd152fc63cc4a67ca29e4647b/drivers/firmware/raspberrypi.c
// This tutorial - https://www.rpi4os.com/part5-framebuffer/
// The official mailbox docs - https://github.com/raspberrypi/firmware/wiki/Mailboxes

//Turns out this peripheral needs a non cached memory

use core::mem::size_of;

use super::{PERIPHERALS_BASE_ADDRESS, utils::{compile_time_size_assert, MmioReg32, memory_barrier}};

const MBOX_BASE_ADDRESS:usize = PERIPHERALS_BASE_ADDRESS + 0xB880;
const MBOX_CHANNEL:u32 = 8;     // free channel for communication from the cpu to the core

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

#[repr(C, align(4))]
struct MailboxRegisters{
    read_reg:MmioReg32,
    _res:[u32;5],
    status:MmioReg32,
    _res1:u32,
    write_reg:MmioReg32,
}
compile_time_size_assert!(MailboxRegisters, 0x24);

const MAILBOX_BUFFER_SIZE:usize = 0x100;
#[repr(align(16))]
struct MailboxBuffer([u8;MAILBOX_BUFFER_SIZE]);
#[no_mangle]
#[link_section = ".uncached"]
static mut MAILBOX_UNCACHED_BUFFER:MailboxBuffer = MailboxBuffer([0;MAILBOX_BUFFER_SIZE]);

pub struct Mailbox{
    registers:&'static mut MailboxRegisters,
    uncached_buffer:&'static mut [u8]
}

impl Mailbox{
    const STATUS_EMPTY:u32  = 0x4000_0000;
    const STATUS_FULL:u32   = 0x8000_0000;

    pub(super) fn new()->Mailbox{
        let registers = unsafe{&mut *(MBOX_BASE_ADDRESS as *mut MailboxRegisters)};
        let uncached_buffer:&'static mut [u8] = unsafe{&mut MAILBOX_UNCACHED_BUFFER.0};
        return Mailbox { registers, uncached_buffer };
    }

    pub fn call<const DATA_LEN:usize>(&mut self, tag:u32, data:[u32;DATA_LEN])->[u32;DATA_LEN]{
        if size_of::<Message<DATA_LEN>>() > self.uncached_buffer.len() {
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