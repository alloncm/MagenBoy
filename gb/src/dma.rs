use std::ptr::write_volatile;

use libc::{c_void, c_int};

fn libc_abort(message:&str){
    std::io::Result::<&str>::Err(std::io::Error::last_os_error()).expect(message);
}

macro_rules! decl_write_volatile_field{
    ($function_name:ident, $field_name:ident) =>{
        #[inline] unsafe fn $function_name(&mut self,value:u32){
            std::ptr::write_volatile(&mut self.$field_name , value);
        }
    }
}

macro_rules! decl_read_volatile_field{
    ($function_name:ident, $field_name:ident) =>{
        #[inline] unsafe fn $function_name(&mut self)->u32{
            std::ptr::read_volatile(&self.$field_name)
        }
    }
}

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
        let ret = unsafe{libc::ioctl(self.mbox_fd, Self::MAILBOX_IOCTL_PROPERTY, raw_message)};
        if ret < 0{
            libc_abort("Error in ioctl call");
        }

        return message.data[0];
    }
}

struct DmaMemory{
    virtual_address_ptr:usize,
    bus_address:u32,
    mailbox_memory_handle:u32,
    size:u32
}

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

pub struct DmaTransferer<const CHUNK_SIZE:usize, const NUM_CHUNKS:usize>{
    tx_dma:*mut DmaRegistersAccess,
    rx_dma:*mut DmaRegistersAccess,
    mbox:Mailbox,
    tx_control_block_memory:DmaMemory,
    rx_control_block_memory:DmaMemory,
    source_buffer_memory:DmaMemory,
    dma_data_memory:DmaMemory,
    dma_const_data_memory:DmaMemory,
    tx_channel_number:u8,
    rx_channel_number:u8
}

impl<const CHUNK_SIZE:usize, const NUM_CHUNKS:usize> DmaTransferer<CHUNK_SIZE, NUM_CHUNKS>{
    const BCM2835_DMA0_BASE:usize = 0x7_000;

    const DMA_CS_RESET:u32 = 1 << 31;
    const DMA_CS_END:u32 = 1 << 1;
    const DMA_CS_ACTIVE:u32 = 1;

    const DMA_TI_SRC_DREQ:u32 = 1 << 10;
    const DMA_TI_SRC_INC:u32 = 1 << 8;
    const DMA_TI_DEST_IGNORE:u32 = 1 << 7;
    const DMA_TI_DEST_DREQ:u32 = 1 << 6;
    const DMA_TI_DEST_INC:u32 = 1 << 4;
    const DMA_TI_WAIT_RESP:u32 = 1 << 3;

    const DMA_DMA0_CB_PHYS_ADDRESS:u32 = 0x7E00_7000;
    const fn dma_ti_permap(peripherial_mapping:u8)->u32{(peripherial_mapping as u32) << 16}

    pub fn new(bcm2835:*mut c_void, tx_channel_number:u8, rx_channel_number:u8, mem_fd:c_int)->Self{
        let mbox = Mailbox::new();
        let tx_registers = unsafe{bcm2835.add(Self::BCM2835_DMA0_BASE + (tx_channel_number as usize * 0x100)) as *mut DmaRegistersAccess };
        let rx_registers = unsafe{bcm2835.add(Self::BCM2835_DMA0_BASE + (rx_channel_number as usize * 0x100)) as *mut DmaRegistersAccess };
        let dma_tx_control_block_memory = Self::allocate_dma_memory(&mbox, std::mem::size_of::<DmaControlBlock>() as u32 * 4 * NUM_CHUNKS as u32, mem_fd);
        let dma_rx_control_block_memory = Self::allocate_dma_memory(&mbox, std::mem::size_of::<DmaControlBlock>() as u32 * NUM_CHUNKS as u32, mem_fd);
        let dma_source_buffer_memory = Self::allocate_dma_memory(&mbox, (NUM_CHUNKS * CHUNK_SIZE) as u32, mem_fd);
        let dma_data_memory = Self::allocate_dma_memory(&mbox, (std::mem::size_of::<u32>() * NUM_CHUNKS) as u32, mem_fd);
        let dma_const_data_memory = Self::allocate_dma_memory(&mbox, (std::mem::size_of::<u32>() * 2) as u32, mem_fd);

        let dma_enable_register = unsafe{bcm2835.add(Self::BCM2835_DMA0_BASE + 0xFF0) as *mut u32};

        unsafe{
            // setup constant data
            let ptr = dma_const_data_memory.virtual_address_ptr as *mut u32;
            write_volatile(ptr, 0x100); // spi_dma enable
            write_volatile(ptr.add(1), Self::DMA_CS_ACTIVE | Self::DMA_CS_END);

            // enable the rx & tx dma channels
            write_volatile(dma_enable_register, *dma_enable_register | 1 << tx_channel_number | 1<< rx_channel_number);

            //reset the dma channels
            (*tx_registers).write_cs(Self::DMA_CS_RESET);
            (*rx_registers).write_cs(Self::DMA_CS_RESET);

            // memset the memory
            std::ptr::write_bytes(dma_rx_control_block_memory.virtual_address_ptr as *mut u8, 0, dma_rx_control_block_memory.size as usize);
            std::ptr::write_bytes(dma_tx_control_block_memory.virtual_address_ptr as *mut u8, 0, dma_tx_control_block_memory.size as usize);
            std::ptr::write_bytes(dma_source_buffer_memory.virtual_address_ptr as *mut u8, 0, dma_source_buffer_memory.size as usize);
            std::ptr::write_bytes(dma_data_memory.virtual_address_ptr as *mut u8, 0, dma_data_memory.size as usize);
        }

        Self { 
            tx_dma: tx_registers,
            rx_dma: rx_registers,
            mbox,
            rx_control_block_memory:dma_rx_control_block_memory,
            tx_control_block_memory:dma_tx_control_block_memory,
            source_buffer_memory:dma_source_buffer_memory,
            dma_data_memory,
            rx_channel_number,
            tx_channel_number,
            dma_const_data_memory
        }
    }


    const DMA_SPI_CS_PHYS_ADDRESS:u32 = 0x7E20_4000;

    pub fn dma_transfer<const SIZE:usize>(&mut self, data:&[u8; SIZE], tx_peripherial_mapping:u8, tx_physical_destination_address:u32, rx_peripherial_mapping:u8, rx_physical_destination_address:u32){
        if SIZE != NUM_CHUNKS * CHUNK_SIZE{
            std::panic!("bad SIZE param");
        }

        unsafe{
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.source_buffer_memory.virtual_address_ptr as *mut u8, SIZE);

            let mut rx_control_block = &mut *(self.rx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock);
            rx_control_block.write_ti(Self::dma_ti_permap(rx_peripherial_mapping) | Self::DMA_TI_SRC_DREQ | Self::DMA_TI_DEST_IGNORE);
            rx_control_block.write_source_ad(rx_physical_destination_address);
            rx_control_block.write_dest_ad(0);
            rx_control_block.write_txfr_len(CHUNK_SIZE as u32 - 4);       // without the 4 byte header
            rx_control_block.write_nextconbk(0);

            let tx_control_block = &mut *(self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock);
            tx_control_block.write_ti(Self::dma_ti_permap(tx_peripherial_mapping) | Self::DMA_TI_DEST_DREQ | Self::DMA_TI_SRC_INC | Self::DMA_TI_WAIT_RESP);
            tx_control_block.write_source_ad(self.source_buffer_memory.bus_address);
            tx_control_block.write_dest_ad(tx_physical_destination_address);
            tx_control_block.write_txfr_len(CHUNK_SIZE as u32);
            tx_control_block.write_nextconbk(0);

            for i in 1..NUM_CHUNKS{
                let tx_cb_index = i * 4;
                let tx_control_block = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(tx_cb_index));
                tx_control_block.write_ti(Self::dma_ti_permap(tx_peripherial_mapping) | Self::DMA_TI_DEST_DREQ | Self::DMA_TI_SRC_INC | Self::DMA_TI_WAIT_RESP);
                tx_control_block.write_source_ad(self.source_buffer_memory.bus_address + (i * CHUNK_SIZE) as u32);
                tx_control_block.write_dest_ad(tx_physical_destination_address);
                tx_control_block.write_txfr_len(CHUNK_SIZE as u32);
                tx_control_block.write_nextconbk(0);

                let set_dma_tx_address = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(tx_cb_index + 1));
                let disable_dma_tx_address = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(tx_cb_index + 2));
                let start_dma_tx_address = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(tx_cb_index + 3));

                rx_control_block.write_nextconbk(self.tx_control_block_memory.bus_address + ((tx_cb_index + 1) * std::mem::size_of::<DmaControlBlock>()) as u32);

                write_volatile((self.dma_data_memory.virtual_address_ptr as *mut u32).add(i), self.tx_control_block_memory.bus_address + (tx_cb_index * std::mem::size_of::<DmaControlBlock>()) as u32);

                set_dma_tx_address.write_ti(Self::DMA_TI_SRC_INC | Self::DMA_TI_DEST_INC | Self::DMA_TI_WAIT_RESP);
                set_dma_tx_address.write_source_ad(self.dma_data_memory.bus_address + (i as u32 * 4));
                set_dma_tx_address.write_dest_ad(Self::DMA_DMA0_CB_PHYS_ADDRESS + (self.tx_channel_number as u32 * 0x100) + 4);  // channel control block address register
                set_dma_tx_address.write_txfr_len(4);
                set_dma_tx_address.write_nextconbk(self.tx_control_block_memory.bus_address + ((tx_cb_index + 2) * std::mem::size_of::<DmaControlBlock>()) as u32);


                disable_dma_tx_address.write_ti(Self::DMA_TI_SRC_INC | Self::DMA_TI_DEST_INC | Self::DMA_TI_WAIT_RESP);
                disable_dma_tx_address.write_source_ad(self.dma_const_data_memory.bus_address);
                disable_dma_tx_address.write_dest_ad(Self::DMA_SPI_CS_PHYS_ADDRESS);
                disable_dma_tx_address.write_txfr_len(4);
                disable_dma_tx_address.write_nextconbk(self.tx_control_block_memory.bus_address + ((tx_cb_index + 3) * std::mem::size_of::<DmaControlBlock>()) as u32);

                
                start_dma_tx_address.write_ti(Self::DMA_TI_SRC_INC | Self::DMA_TI_DEST_INC | Self::DMA_TI_WAIT_RESP);
                start_dma_tx_address.write_source_ad(self.dma_const_data_memory.bus_address + 4);
                start_dma_tx_address.write_dest_ad(Self::DMA_DMA0_CB_PHYS_ADDRESS + (self.tx_channel_number as u32 * 0x100) as u32);
                start_dma_tx_address.write_txfr_len(4);
                start_dma_tx_address.write_nextconbk(self.rx_control_block_memory.bus_address + (i * std::mem::size_of::<DmaControlBlock>()) as u32);


                rx_control_block = &mut *((self.rx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(i));
                rx_control_block.write_ti(Self::dma_ti_permap(rx_peripherial_mapping) | Self::DMA_TI_SRC_DREQ | Self::DMA_TI_DEST_IGNORE);
                rx_control_block.write_source_ad(rx_physical_destination_address);
                rx_control_block.write_dest_ad(0);
                rx_control_block.write_txfr_len(CHUNK_SIZE as u32 - 4);       // without the 4 byte header
                rx_control_block.write_nextconbk(0);
            }

            
            (*self.tx_dma).write_conblk_ad(self.tx_control_block_memory.bus_address);
            (*self.rx_dma).write_conblk_ad(self.rx_control_block_memory.bus_address);

            // Starting the dma transfer
            (*self.tx_dma).write_cs(Self::DMA_CS_ACTIVE | Self::DMA_CS_END);
            (*self.rx_dma).write_cs(Self::DMA_CS_ACTIVE | Self::DMA_CS_END);

            
        }
    }

    pub fn wait_for_dma_transfer(&self){
        unsafe{
            // Wait for the last trasfer to end
            while (*self.tx_dma).read_cs() & Self::DMA_CS_ACTIVE != 0 {
                // Self::sleep_ms(250);
                // log::info!("Waiting for the tx channel");
            }
            while (*self.rx_dma).read_cs() & Self::DMA_CS_ACTIVE != 0 {
                // Self::sleep_ms(250);
                // log::info!("Waiting for the rx channel");
            }
        }
    }

    fn sleep_ms(milliseconds_to_sleep:u64){
        std::thread::sleep(std::time::Duration::from_millis(milliseconds_to_sleep));
    }

    const MEM_ALLOC_FLAG_DIRECT:usize = 1 << 2;
    const MEM_ALLOC_FLAG_COHERENT:usize = 1 << 3;
    // This function converts the from the bus address of the SDRAM uncached memory to the arm physical address
    // Notice that supposed to work only for this type of memory
    const fn bus_to_phys(bus_address:u32)->u32{bus_address & !0xC000_0000}

    fn allocate_dma_memory(mbox:&Mailbox, size:u32, mem_fd:c_int)->DmaMemory{
        let flags = (Self::MEM_ALLOC_FLAG_COHERENT | Self::MEM_ALLOC_FLAG_DIRECT) as u32;
        let handle = mbox.send_command(0x3000C, [size, 4096, flags]);

        let bus_address = mbox.send_command(0x3000D, [handle]);
        let virtual_address = unsafe{libc::mmap(
            std::ptr::null_mut(),
            size as libc::size_t,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            mem_fd,
            Self::bus_to_phys(bus_address) as libc::off_t
        )};

        return DmaMemory { virtual_address_ptr: virtual_address as usize, bus_address, mailbox_memory_handle:handle, size }
    }
}


