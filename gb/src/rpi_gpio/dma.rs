use std::ptr::{write_volatile, read_volatile};

use bcm_host::BcmHost;
use libc::{c_void, c_int};

use super::{*, ili9341_controller::SPI_BUFFER_SIZE};

// Mailbox messages need to be 16 byte alligned
#[repr(C, align(16))]
struct MailboxMessage<const PAYLOAD_SIZE:usize>{
    length:u32,
    request:u32,
    tag:u32,
    buffer_length:u32,
    data_length:u32,            // not sure if neccessary
    data:[u32;PAYLOAD_SIZE],
    message_end_indicator:u32
}

impl<const PAYLOAD_SIZE:usize> MailboxMessage<PAYLOAD_SIZE>{
    fn new(tag:u32, data:[u32;PAYLOAD_SIZE])->Self{
        Self{
            length:std::mem::size_of::<Self>() as u32,
            request:0,
            tag,
            buffer_length:(std::mem::size_of::<u32>()*PAYLOAD_SIZE) as u32,
            data_length:(std::mem::size_of::<u32>()*PAYLOAD_SIZE) as u32,
            data,
            message_end_indicator:0
        }
    }
}

// Docs - https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface
struct Mailbox{
    mbox_fd: c_int,
}

impl Mailbox{
    const MAILBOX_IOCTL_PROPERTY:libc::c_ulong = nix::request_code_readwrite!(100, 0, std::mem::size_of::<*mut libc::c_void>());

    fn new()->Self{
        let fd = unsafe{libc::open(std::ffi::CStr::from_bytes_with_nul(b"/dev/vcio\0").unwrap().as_ptr(), 0)};
        if fd < 0{
            std::panic!("Error while opening vc mailbox");
        }

        Self { mbox_fd: fd }
    }

    fn send_command<const SIZE:usize>(&self, tag:u32, data:[u32;SIZE])->u32{
        let mut message = MailboxMessage::<SIZE>::new(tag, data);
        return self.send_message(&mut message);
    }

    fn send_message<const SIZE:usize>(&self, message:&mut MailboxMessage<SIZE>)->u32{
        let raw_message = message as *mut MailboxMessage<SIZE> as *mut c_void;
        let ret = unsafe{
            // Using libc::ioctl and not nix high level abstraction over it cause Im sending a *void and not more 
            // concrete type and the nix macro will mess the types for us. I belive it could work with nix after some modification 
            // of the way Im handling this but Im leaving this as it for now. sorry!
            libc::ioctl(self.mbox_fd, Self::MAILBOX_IOCTL_PROPERTY, raw_message)
        };
        if ret < 0{
            libc_abort("Error in ioctl call");
        }

        // The return value of the command is located at the first int in the data section (for more info see the Mailbox docs)
        return message.data[0];
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


// using GpuMemory cause I need a memory that is not cached by the cpu caches (L1, L2)
struct GpuMemory{
    virtual_address_ptr:usize,
    bus_address:u32,
    mailbox_memory_handle:u32,
    size:u32
}

impl GpuMemory{
    const MEM_ALLOC_FLAG_DIRECT:usize = 1 << 2;
    const MEM_ALLOC_FLAG_COHERENT:usize = 1 << 3;
    const ALLOCATE_MEMORY_TAG:u32 = 0x3000C;
    const LOCK_MEMORY_TAG:u32 = 0x3000D;
    const UNLOCK_MEMORY_TAG:u32 = 0x3000E;
    const RELEASE_MEMORY_TAG:u32 = 0x3000E;
    const PAGE_SIZE:u32 = 4096;

    // This function converts the from the bus address of the SDRAM uncached memory to the arm physical address
    // Notice that supposed to work only for this type of memory
    const fn bus_to_phys(bus_address:u32)->u32{bus_address & !0xC000_0000}

    // Using the Mailbox interface to allocate memory on the gpu
    fn allocate(mbox:&Mailbox, size:u32, mem_fd:c_int)->GpuMemory{
        let flags = (Self::MEM_ALLOC_FLAG_COHERENT | Self::MEM_ALLOC_FLAG_DIRECT) as u32;
        let handle = mbox.send_command(Self::ALLOCATE_MEMORY_TAG, [size, Self::PAGE_SIZE, flags]);

        let bus_address = mbox.send_command(Self::LOCK_MEMORY_TAG, [handle]);
        let virtual_address = unsafe{libc::mmap(
            std::ptr::null_mut(),
            size as libc::size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            mem_fd,
            Self::bus_to_phys(bus_address) as libc::off_t
        )};

        return GpuMemory { virtual_address_ptr: virtual_address as usize, bus_address, mailbox_memory_handle:handle, size }
    }

    fn release(&self, mbox:&Mailbox){
        unsafe{
            let result = libc::munmap(self.virtual_address_ptr as *mut c_void, self.size as libc::size_t);
            if result != 0 {
                libc_abort("Error while trying to un map gpu memory");
            }
        }
        let status = mbox.send_command(Self::UNLOCK_MEMORY_TAG, [self.mailbox_memory_handle]);
        if status != 0{
            std::panic!("Error while trying to unlock gpu memory using mailbox");
        }
        let status = mbox.send_command(Self::RELEASE_MEMORY_TAG, [self.mailbox_memory_handle]);
        if status != 0{
            std::panic!("Error while to release gpu memory using mailbox");
        }
    }
}

// The DMA control block registers are in a 32 byte alligned addresses so the stracture mapping them needs to be as well
// in order for me to cast some bytes to this stuct (at least I think so)
// Not valid for DMA4 channels
#[repr(C, align(32))]
struct DmaControlBlock{
    transfer_information:u32,
    source_address:u32,
    destination_address:u32,
    trasnfer_length:u32,
    _stride:u32,                 // Not avalibale on the lite channels
    next_control_block_address:u32,
    _reserved:[u32;2]
}

impl DmaControlBlock{
    decl_write_volatile_field!(write_ti, transfer_information);
    decl_write_volatile_field!(write_source_ad, source_address);
    decl_write_volatile_field!(write_dest_ad, destination_address);
    decl_write_volatile_field!(write_txfr_len, trasnfer_length);
    decl_write_volatile_field!(write_nextconbk, next_control_block_address);
}


// Since Im casting an arbitary pointer to this struct it must be alligned by 4 bytes (with no gaps as well)
#[repr(C, align(4))]
struct DmaRegistersAccess{
    control_status:u32,
    control_block_address:u32,
    control_block:DmaControlBlock,
    debug:u32
}

impl DmaRegistersAccess{
    decl_write_volatile_field!(write_cs, control_status);
    decl_read_volatile_field!(read_cs, control_status);
    decl_write_volatile_field!(write_conblk_ad, control_block_address);
}

pub struct DmaSpiTransferer{
    tx_dma:*mut DmaRegistersAccess,
    rx_dma:*mut DmaRegistersAccess,
    mbox:Mailbox,
    tx_control_block_memory:GpuMemory,
    rx_control_block_memory:GpuMemory,
    source_buffer_memory:GpuMemory,
    dma_dynamic_memory:GpuMemory,
    dma_constant_memory:GpuMemory,
    dma_enable_register_ptr:*mut u32,
}

impl DmaSpiTransferer{
    const BCM_DMA0_OFFSET:usize = 0x7_000;
    const BCM_DMA_ENABLE_REGISTER_OFFSET:usize = Self::BCM_DMA0_OFFSET + 0xFF;
    const DMA_CHANNEL_REGISTERS_SIZE:usize = 0x100;

    const DMA_CS_RESET:u32 = 1 << 31;
    const DMA_CS_END:u32 = 1 << 1;
    const DMA_CS_ACTIVE:u32 = 1;

    const DMA_TI_SRC_DREQ:u32 = 1 << 10;
    const DMA_TI_SRC_INC:u32 = 1 << 8;
    const DMA_TI_DEST_IGNORE:u32 = 1 << 7;
    const DMA_TI_DEST_DREQ:u32 = 1 << 6;
    const DMA_TI_DEST_INC:u32 = 1 << 4;
    const DMA_TI_WAIT_RESP:u32 = 1 << 3;

    const DMA_DMA0_CS_PHYS_ADDRESS:u32 = 0x7E00_7000;
    const DMA_DMA0_CONBLK_AD_PHYS_ADDRESS:u32 = 0x7E00_7004;
    const fn dma_ti_permap(peripherial_mapping:u8)->u32{(peripherial_mapping as u32) << 16}

    const DMA_CONTROL_BLOCKS_PER_TRANSFER:u32 = 4;
    const DMA_CONSTANT_MEMORY_SIZE:u32 = (std::mem::size_of::<u32>() * 2) as u32;
    const DMA_SPI_HEADER_SIZE:u32 = std::mem::size_of::<u32>() as u32;
    const RX_CHANNEL_NUMBER:u8 = 7;
    const TX_CHANNEL_NUMBER:u8 = 1;

    const MAX_DMA_SPI_TRANSFER:usize = 0xFFE0;  // must be smaller than max u16 and better be alligned for 32 bytes
    const DMA_SPI_NUM_CHUNKS:usize = (SPI_BUFFER_SIZE / Self::MAX_DMA_SPI_TRANSFER) + ((SPI_BUFFER_SIZE % Self::MAX_DMA_SPI_TRANSFER) != 0) as usize;
    const DMA_SPI_CHUNK_SIZE:usize = (SPI_BUFFER_SIZE / Self::DMA_SPI_NUM_CHUNKS) + 4;
    const DMA_SPI_TRANSFER_SIZE:usize = Self::DMA_SPI_CHUNK_SIZE * Self::DMA_SPI_NUM_CHUNKS;
    const DMA_TI_PERMAP_SPI_TX:u8 = 6;
    const DMA_TI_PERMAP_SPI_RX:u8 = 7;

    const DMA_SPI_CS_PHYS_ADDRESS:u32 = 0x7E20_4000;
    const DMA_SPI_FIFO_PHYS_ADDRESS:u32 = 0x7E20_4004;

    pub fn new(bcm_host:&BcmHost, spi_enable_dma_flag:u32)->Self{
        let mbox = Mailbox::new();
        let tx_registers = bcm_host.get_ptr(Self::BCM_DMA0_OFFSET + (Self::TX_CHANNEL_NUMBER as usize * Self::DMA_CHANNEL_REGISTERS_SIZE)) as *mut DmaRegistersAccess;
        let rx_registers = bcm_host.get_ptr(Self::BCM_DMA0_OFFSET + (Self::RX_CHANNEL_NUMBER as usize * Self::DMA_CHANNEL_REGISTERS_SIZE)) as *mut DmaRegistersAccess;
        
        let dma_tx_control_block_memory = GpuMemory::allocate(&mbox, 
            std::mem::size_of::<DmaControlBlock>() as u32 * Self::DMA_CONTROL_BLOCKS_PER_TRANSFER * Self::DMA_SPI_CHUNK_SIZE as u32, 
            bcm_host.get_fd()
        );
        let dma_rx_control_block_memory = GpuMemory::allocate(&mbox, 
            std::mem::size_of::<DmaControlBlock>() as u32 * Self::DMA_SPI_NUM_CHUNKS as u32, 
            bcm_host.get_fd()
        );
        let dma_source_buffer_memory = GpuMemory::allocate(&mbox, (Self::DMA_SPI_TRANSFER_SIZE) as u32, bcm_host.get_fd());
        let dma_dynamic_memory = GpuMemory::allocate(&mbox, (std::mem::size_of::<u32>() * Self::DMA_SPI_NUM_CHUNKS) as u32, bcm_host.get_fd());
        let dma_constant_memory = GpuMemory::allocate(&mbox, Self::DMA_CONSTANT_MEMORY_SIZE, bcm_host.get_fd());

        let dma_enable_register = bcm_host.get_ptr(Self::BCM_DMA_ENABLE_REGISTER_OFFSET) as *mut u32;

        unsafe{
            // setup constant data
            let ptr = dma_constant_memory.virtual_address_ptr as *mut u32;
            write_volatile(ptr, spi_enable_dma_flag);                                       // this int enable spi with dma
            write_volatile(ptr.add(1), Self::DMA_CS_ACTIVE | Self::DMA_CS_END);      // this int starts the dma (set active and wrtie to end to reset it)

            // enable the rx & tx dma channels
            write_volatile(dma_enable_register, *dma_enable_register | 1 << Self::TX_CHANNEL_NUMBER | 1<< Self::RX_CHANNEL_NUMBER);

            //reset the dma channels
            (*tx_registers).write_cs(Self::DMA_CS_RESET);
            (*rx_registers).write_cs(Self::DMA_CS_RESET);

            // memset the memory
            std::ptr::write_bytes(dma_rx_control_block_memory.virtual_address_ptr as *mut u8, 0, dma_rx_control_block_memory.size as usize);
            std::ptr::write_bytes(dma_tx_control_block_memory.virtual_address_ptr as *mut u8, 0, dma_tx_control_block_memory.size as usize);
            std::ptr::write_bytes(dma_source_buffer_memory.virtual_address_ptr as *mut u8, 0, dma_source_buffer_memory.size as usize);
            std::ptr::write_bytes(dma_dynamic_memory.virtual_address_ptr as *mut u8, 0, dma_dynamic_memory.size as usize);
        }

        let mut dma_controller = Self { 
            tx_dma: tx_registers,
            rx_dma: rx_registers,
            mbox,
            rx_control_block_memory:dma_rx_control_block_memory,
            tx_control_block_memory:dma_tx_control_block_memory,
            source_buffer_memory:dma_source_buffer_memory,
            dma_dynamic_memory,
            dma_constant_memory,
            dma_enable_register_ptr:dma_enable_register
        };

        unsafe{dma_controller.init_dma_control_blocks()};

        return dma_controller;
    }

    unsafe fn init_dma_control_blocks(&mut self) {
        let mut rx_control_block = &mut *(self.rx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock);
        rx_control_block.write_ti(Self::dma_ti_permap(Self::DMA_TI_PERMAP_SPI_RX) | Self::DMA_TI_SRC_DREQ | Self::DMA_TI_DEST_IGNORE);
        rx_control_block.write_source_ad(Self::DMA_SPI_FIFO_PHYS_ADDRESS);
        rx_control_block.write_dest_ad(0);
        rx_control_block.write_txfr_len(Self::DMA_SPI_CHUNK_SIZE as u32 - Self::DMA_SPI_HEADER_SIZE); // without the 4 byte header
        rx_control_block.write_nextconbk(0);

        let tx_control_block = &mut *(self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock);
        tx_control_block.write_ti(Self::dma_ti_permap(Self::DMA_TI_PERMAP_SPI_TX) | Self::DMA_TI_DEST_DREQ | Self::DMA_TI_SRC_INC | Self::DMA_TI_WAIT_RESP);
        tx_control_block.write_source_ad(self.source_buffer_memory.bus_address);
        tx_control_block.write_dest_ad(Self::DMA_SPI_FIFO_PHYS_ADDRESS);
        tx_control_block.write_txfr_len(Self::DMA_SPI_CHUNK_SIZE as u32);
        tx_control_block.write_nextconbk(0);
        for i in 1..Self::DMA_SPI_NUM_CHUNKS{
            let tx_cb_index = i * Self::DMA_CONTROL_BLOCKS_PER_TRANSFER as usize;
            let set_tx_cb_index = tx_cb_index + 1;
            let disable_tx_cb_index = set_tx_cb_index + 1;
            let start_tx_cb_index = disable_tx_cb_index + 1;
            
            let tx_control_block = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(tx_cb_index));
            tx_control_block.write_ti(Self::dma_ti_permap(Self::DMA_TI_PERMAP_SPI_TX) | Self::DMA_TI_DEST_DREQ | Self::DMA_TI_SRC_INC | Self::DMA_TI_WAIT_RESP);
            tx_control_block.write_source_ad(self.source_buffer_memory.bus_address + (i * Self::DMA_SPI_CHUNK_SIZE) as u32);
            tx_control_block.write_dest_ad(Self::DMA_SPI_FIFO_PHYS_ADDRESS);
            tx_control_block.write_txfr_len(Self::DMA_SPI_CHUNK_SIZE as u32);
            tx_control_block.write_nextconbk(0);

            let set_dma_tx_address = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(set_tx_cb_index));
            let disable_dma_tx_address = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(disable_tx_cb_index));
            let start_dma_tx_address = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(start_tx_cb_index));

            rx_control_block.write_nextconbk(self.tx_control_block_memory.bus_address + (set_tx_cb_index * std::mem::size_of::<DmaControlBlock>()) as u32);

            write_volatile((self.dma_dynamic_memory.virtual_address_ptr as *mut u32).add(i), self.tx_control_block_memory.bus_address + (tx_cb_index * std::mem::size_of::<DmaControlBlock>()) as u32);

            set_dma_tx_address.write_ti(Self::DMA_TI_SRC_INC | Self::DMA_TI_DEST_INC | Self::DMA_TI_WAIT_RESP);
            set_dma_tx_address.write_source_ad(self.dma_dynamic_memory.bus_address + (i as u32 * Self::DMA_CONTROL_BLOCKS_PER_TRANSFER));
            set_dma_tx_address.write_dest_ad(Self::DMA_DMA0_CONBLK_AD_PHYS_ADDRESS + (Self::TX_CHANNEL_NUMBER as u32 * Self::DMA_CHANNEL_REGISTERS_SIZE as u32));  // channel control block address register
            set_dma_tx_address.write_txfr_len(std::mem::size_of::<u32>() as u32);
            set_dma_tx_address.write_nextconbk(self.tx_control_block_memory.bus_address + (disable_tx_cb_index * std::mem::size_of::<DmaControlBlock>()) as u32);


            disable_dma_tx_address.write_ti(Self::DMA_TI_SRC_INC | Self::DMA_TI_DEST_INC | Self::DMA_TI_WAIT_RESP);
            disable_dma_tx_address.write_source_ad(self.dma_constant_memory.bus_address);
            disable_dma_tx_address.write_dest_ad(Self::DMA_SPI_CS_PHYS_ADDRESS);
            disable_dma_tx_address.write_txfr_len(std::mem::size_of::<u32>() as u32);
            disable_dma_tx_address.write_nextconbk(self.tx_control_block_memory.bus_address + (start_tx_cb_index * std::mem::size_of::<DmaControlBlock>()) as u32);

    
            start_dma_tx_address.write_ti(Self::DMA_TI_SRC_INC | Self::DMA_TI_DEST_INC | Self::DMA_TI_WAIT_RESP);
            start_dma_tx_address.write_source_ad(self.dma_constant_memory.bus_address + (i * std::mem::size_of::<u32>()) as u32);
            start_dma_tx_address.write_dest_ad(Self::DMA_DMA0_CS_PHYS_ADDRESS + (Self::TX_CHANNEL_NUMBER as u32 * Self::DMA_CHANNEL_REGISTERS_SIZE as u32));
            start_dma_tx_address.write_txfr_len(std::mem::size_of::<u32>() as u32);
            start_dma_tx_address.write_nextconbk(self.rx_control_block_memory.bus_address + (i * std::mem::size_of::<DmaControlBlock>()) as u32);


            rx_control_block = &mut *((self.rx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(i));
            rx_control_block.write_ti(Self::dma_ti_permap(Self::DMA_TI_PERMAP_SPI_RX) | Self::DMA_TI_SRC_DREQ | Self::DMA_TI_DEST_IGNORE);
            rx_control_block.write_source_ad(Self::DMA_SPI_FIFO_PHYS_ADDRESS);
            rx_control_block.write_dest_ad(0);
            rx_control_block.write_txfr_len(Self::DMA_SPI_CHUNK_SIZE as u32 - Self::DMA_SPI_HEADER_SIZE);       // without the 4 byte header
            rx_control_block.write_nextconbk(0);
        }
    }

    pub fn start_dma_transfer(&mut self, data:&[u8; SPI_BUFFER_SIZE], transfer_active_flag:u8){        
        unsafe{
            let data_len = Self::DMA_SPI_CHUNK_SIZE - Self::DMA_SPI_HEADER_SIZE as usize;  // Removing the first 4 bytes from this length param
            let header = [transfer_active_flag, 0, (data_len & 0xFF) as u8,  /*making sure this is little endian order*/ (data_len >> 8) as u8];

            let chunks = data.chunks_exact(Self::DMA_SPI_CHUNK_SIZE - Self::DMA_SPI_HEADER_SIZE as usize);
            let mut array:[u8;Self::DMA_SPI_CHUNK_SIZE * Self::DMA_SPI_NUM_CHUNKS] = [0;Self::DMA_SPI_CHUNK_SIZE * Self::DMA_SPI_NUM_CHUNKS];
            let mut i = 0;
            for chunk in chunks{
                std::ptr::copy_nonoverlapping(header.as_ptr(), array.as_mut_ptr().add(i * Self::DMA_SPI_CHUNK_SIZE), 4);
                std::ptr::copy_nonoverlapping(chunk.as_ptr(), array.as_mut_ptr().add(4 + (i * Self::DMA_SPI_CHUNK_SIZE)), Self::DMA_SPI_CHUNK_SIZE - 4);
                i += 1;
            }

            std::ptr::copy_nonoverlapping(array.as_ptr(), self.source_buffer_memory.virtual_address_ptr as *mut u8, array.len());
            
            (*self.tx_dma).write_conblk_ad(self.tx_control_block_memory.bus_address);
            (*self.rx_dma).write_conblk_ad(self.rx_control_block_memory.bus_address);

            // Starting the dma transfer
            (*self.tx_dma).write_cs(Self::DMA_CS_ACTIVE | Self::DMA_CS_END);
            (*self.rx_dma).write_cs(Self::DMA_CS_ACTIVE | Self::DMA_CS_END);
        }
    }

    pub fn end_dma_transfer(&self){
        const TIME_TO_ABORT_AS_MICRO:i32 = 1_000_000;
        unsafe{
            // Wait for the last trasfer to end
            let mut counter = 0;
            while (*self.tx_dma).read_cs() & Self::DMA_CS_ACTIVE != 0 {
                Self::sleep_us(1);
                counter += 1;
                if counter > TIME_TO_ABORT_AS_MICRO{
                    std::panic!("ERROR! tx dma channel is not responding, a reboot is suggested");
                }
            }
            while (*self.rx_dma).read_cs() & Self::DMA_CS_ACTIVE != 0 {
                Self::sleep_us(1);
                counter += 1;
                if counter > TIME_TO_ABORT_AS_MICRO{
                    std::panic!("ERROR! rx dma channel is not responding, a reboot is suggested");
                }
            }
        }
    }

    fn sleep_us(microseconds_to_sleep:u64){
        std::thread::sleep(std::time::Duration::from_micros(microseconds_to_sleep));
    }
}

impl Drop for DmaSpiTransferer{
    fn drop(&mut self) {
        // reset the dma channels before releasing the memory
        unsafe{
            // reset the dma channels
            (*self.tx_dma).write_cs(Self::DMA_CS_RESET);
            (*self.rx_dma).write_cs(Self::DMA_CS_RESET);
            // disable the channels I used
            let mask = !((1 << Self::TX_CHANNEL_NUMBER) | (1 << Self::RX_CHANNEL_NUMBER));
            write_volatile(self.dma_enable_register_ptr, read_volatile(self.dma_enable_register_ptr) & mask);
        }

        self.dma_constant_memory.release(&self.mbox);
        self.dma_dynamic_memory.release(&self.mbox);
        self.rx_control_block_memory.release(&self.mbox);
        self.source_buffer_memory.release(&self.mbox);
        self.tx_control_block_memory.release(&self.mbox);
    }
}