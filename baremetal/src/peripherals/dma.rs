use core::ptr::write_volatile;

use super::{utils::{MmioReg32, compile_time_size_assert, memory_barrier}, gpu::GpuMemory, PERIPHERALS_BASE_ADDRESS, spi::SPI_BUFFER_SIZE};

// The DMA control block registers are in a 32 byte alligned addresses so the stracture mapping them needs to be as well
// in order for me to cast some bytes to this stuct (at least I think so)
// Not valid for DMA4 channels
#[repr(C, align(32))]
struct DmaControlBlock{
    transfer_information:MmioReg32,
    source_address:MmioReg32,
    destination_address:MmioReg32,
    transfer_length:MmioReg32,
    _stride:MmioReg32,                 // Not avalibale on the lite channels
    next_control_block_address:MmioReg32,
    _reserved:[u32;2]
}
compile_time_size_assert!(DmaControlBlock, 0x20);

// Since Im casting an arbitary pointer to this struct it must be alligned by 4 bytes (with no gaps as well)
#[repr(C, align(4))]
struct DmaRegistersAccess{
    control_status:MmioReg32,
    control_block_address:MmioReg32,
    transfer_information:MmioReg32,
    source_address:MmioReg32,
    destination_address:MmioReg32,
    transfer_length:MmioReg32,
    _stride:MmioReg32,                 // Not avalibale on the lite channels
    next_control_block_address:MmioReg32,
    debug:MmioReg32
}
compile_time_size_assert!(DmaRegistersAccess, 0x24);

pub struct DmaSpiTransferer{
    tx_dma:&'static mut DmaRegistersAccess,
    rx_dma:&'static mut DmaRegistersAccess,
    tx_control_block_memory:GpuMemory,
    rx_control_block_memory:GpuMemory,
    source_buffer_memory:GpuMemory,
    dma_dynamic_memory:GpuMemory,
    dma_constant_memory:GpuMemory,
    dma_enable_register:&'static mut MmioReg32,
}

impl DmaSpiTransferer{
    const BCM_DMA0_ADDRESS:usize = PERIPHERALS_BASE_ADDRESS + 0x7_000;
    const BCM_DMA_ENABLE_REGISTER_ADDRESS:usize = Self::BCM_DMA0_ADDRESS + 0xFF0;
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
    const DMA_CONSTANT_MEMORY_SIZE:u32 = (core::mem::size_of::<u32>() * 2) as u32;
    const DMA_SPI_HEADER_SIZE:u32 = core::mem::size_of::<u32>() as u32;
    const RX_CHANNEL_NUMBER:u8 = crate::configuration::peripherals::DMA_RX_CHANNEL_NUMBER;
    const TX_CHANNEL_NUMBER:u8 = crate::configuration::peripherals::DMA_TX_CHANNEL_NUMBER;

    const MAX_DMA_SPI_TRANSFER:usize = 0xFFE0;  // must be smaller than max u16 and better be alligned for 32 bytes
    const DMA_SPI_NUM_CHUNKS:usize = (SPI_BUFFER_SIZE / Self::MAX_DMA_SPI_TRANSFER) + ((SPI_BUFFER_SIZE % Self::MAX_DMA_SPI_TRANSFER) != 0) as usize;
    const DMA_SPI_CHUNK_SIZE:usize = (SPI_BUFFER_SIZE / Self::DMA_SPI_NUM_CHUNKS) + 4;
    const DMA_SPI_TRANSFER_SIZE:usize = Self::DMA_SPI_CHUNK_SIZE * Self::DMA_SPI_NUM_CHUNKS;
    const DMA_TI_PERMAP_SPI_TX:u8 = 6;
    const DMA_TI_PERMAP_SPI_RX:u8 = 7;

    const DMA_SPI0_CS_PHYS_ADDRESS:u32 = 0x7E20_4000;
    const DMA_SPI0_FIFO_PHYS_ADDRESS:u32 = 0x7E20_4004;

    pub fn new(spi_enable_dma_flag:u32)->Self{
        let tx_registers = unsafe{&mut *((Self::BCM_DMA0_ADDRESS + (Self::TX_CHANNEL_NUMBER as usize * Self::DMA_CHANNEL_REGISTERS_SIZE)) as *mut DmaRegistersAccess)};
        let rx_registers = unsafe{&mut *((Self::BCM_DMA0_ADDRESS + (Self::RX_CHANNEL_NUMBER as usize * Self::DMA_CHANNEL_REGISTERS_SIZE)) as *mut DmaRegistersAccess)};
        
        let dma_tx_control_block_memory = GpuMemory::allocate(
            core::mem::size_of::<DmaControlBlock>() as u32 * Self::DMA_CONTROL_BLOCKS_PER_TRANSFER * Self::DMA_SPI_CHUNK_SIZE as u32
        );
        let dma_rx_control_block_memory = GpuMemory::allocate(
            core::mem::size_of::<DmaControlBlock>() as u32 * Self::DMA_SPI_NUM_CHUNKS as u32
        );
        let dma_source_buffer_memory = GpuMemory::allocate((Self::DMA_SPI_TRANSFER_SIZE) as u32);
        let dma_dynamic_memory = GpuMemory::allocate((core::mem::size_of::<u32>() * Self::DMA_SPI_NUM_CHUNKS) as u32);
        let dma_constant_memory = GpuMemory::allocate(Self::DMA_CONSTANT_MEMORY_SIZE);

        log::info!("Finish allocate gpu mem");
        let dma_enable_register = unsafe{&mut *(Self::BCM_DMA_ENABLE_REGISTER_ADDRESS as *mut MmioReg32)};

        unsafe{
            // setup constant data
            let ptr = dma_constant_memory.virtual_address_ptr as *mut u32;
            write_volatile(ptr, spi_enable_dma_flag);                                       // this int enable spi with dma
            write_volatile(ptr.add(1), Self::DMA_CS_ACTIVE | Self::DMA_CS_END);      // this int starts the dma (set active and wrtie to end to reset it)

            // enable the rx & tx dma channels
            dma_enable_register.write(dma_enable_register.read() | 1 << Self::TX_CHANNEL_NUMBER | 1 << Self::RX_CHANNEL_NUMBER);

            //reset the dma channels
            tx_registers.control_status.write(Self::DMA_CS_RESET);
            rx_registers.control_status.write(Self::DMA_CS_RESET);

            // memset the memory
            core::ptr::write_bytes(dma_rx_control_block_memory.virtual_address_ptr as *mut u8, 0, dma_rx_control_block_memory.size as usize);
            core::ptr::write_bytes(dma_tx_control_block_memory.virtual_address_ptr as *mut u8, 0, dma_tx_control_block_memory.size as usize);
            core::ptr::write_bytes(dma_source_buffer_memory.virtual_address_ptr as *mut u8, 0, dma_source_buffer_memory.size as usize);
            core::ptr::write_bytes(dma_dynamic_memory.virtual_address_ptr as *mut u8, 0, dma_dynamic_memory.size as usize);
        }

        log::info!("Finish init gpu mem");

        let mut dma_controller = Self { 
            tx_dma: tx_registers,
            rx_dma: rx_registers,
            rx_control_block_memory:dma_rx_control_block_memory,
            tx_control_block_memory:dma_tx_control_block_memory,
            source_buffer_memory:dma_source_buffer_memory,
            dma_dynamic_memory,
            dma_constant_memory,
            dma_enable_register
        };

        unsafe{dma_controller.init_dma_control_blocks()};

        log::info!("Initialized dma contorller");

        return dma_controller;
    }

    unsafe fn init_dma_control_blocks(&mut self) {
        let mut rx_control_block = &mut *(self.rx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock);
        rx_control_block.transfer_information.write(Self::dma_ti_permap(Self::DMA_TI_PERMAP_SPI_RX) | Self::DMA_TI_SRC_DREQ | Self::DMA_TI_DEST_IGNORE);
        rx_control_block.source_address.write(Self::DMA_SPI0_FIFO_PHYS_ADDRESS);
        rx_control_block.destination_address.write(0);
        rx_control_block.transfer_length.write(Self::DMA_SPI_CHUNK_SIZE as u32 - Self::DMA_SPI_HEADER_SIZE); // without the 4 byte header
        rx_control_block.next_control_block_address.write(0);

        let tx_control_block = &mut *(self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock);
        tx_control_block.transfer_information.write(Self::dma_ti_permap(Self::DMA_TI_PERMAP_SPI_TX) | Self::DMA_TI_DEST_DREQ | Self::DMA_TI_SRC_INC | Self::DMA_TI_WAIT_RESP);
        tx_control_block.source_address.write(self.source_buffer_memory.bus_address);
        tx_control_block.destination_address.write(Self::DMA_SPI0_FIFO_PHYS_ADDRESS);
        tx_control_block.transfer_length.write(Self::DMA_SPI_CHUNK_SIZE as u32);
        tx_control_block.next_control_block_address.write(0);
        for i in 1..Self::DMA_SPI_NUM_CHUNKS{
            let tx_cb_index = i * Self::DMA_CONTROL_BLOCKS_PER_TRANSFER as usize;
            let set_tx_cb_index = tx_cb_index + 1;
            let disable_tx_cb_index = set_tx_cb_index + 1;
            let start_tx_cb_index = disable_tx_cb_index + 1;
            
            let tx_control_block = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(tx_cb_index));
            tx_control_block.transfer_information.write(Self::dma_ti_permap(Self::DMA_TI_PERMAP_SPI_TX) | Self::DMA_TI_DEST_DREQ | Self::DMA_TI_SRC_INC | Self::DMA_TI_WAIT_RESP);
            tx_control_block.source_address.write(self.source_buffer_memory.bus_address + (i * Self::DMA_SPI_CHUNK_SIZE) as u32);
            tx_control_block.destination_address.write(Self::DMA_SPI0_FIFO_PHYS_ADDRESS);
            tx_control_block.transfer_length.write(Self::DMA_SPI_CHUNK_SIZE as u32);
            tx_control_block.next_control_block_address.write(0);

            let set_dma_tx_address = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(set_tx_cb_index));
            let disable_dma_tx_address = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(disable_tx_cb_index));
            let start_dma_tx_address = &mut *((self.tx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(start_tx_cb_index));

            rx_control_block.next_control_block_address.write(self.tx_control_block_memory.bus_address + (set_tx_cb_index * core::mem::size_of::<DmaControlBlock>()) as u32);

            write_volatile((self.dma_dynamic_memory.virtual_address_ptr as *mut u32).add(i), self.tx_control_block_memory.bus_address + (tx_cb_index * core::mem::size_of::<DmaControlBlock>()) as u32);

            set_dma_tx_address.transfer_information.write(Self::DMA_TI_SRC_INC | Self::DMA_TI_DEST_INC | Self::DMA_TI_WAIT_RESP);
            set_dma_tx_address.source_address.write(self.dma_dynamic_memory.bus_address + (i as u32 * Self::DMA_CONTROL_BLOCKS_PER_TRANSFER));
            set_dma_tx_address.destination_address.write(Self::DMA_DMA0_CONBLK_AD_PHYS_ADDRESS + (Self::TX_CHANNEL_NUMBER as u32 * Self::DMA_CHANNEL_REGISTERS_SIZE as u32));  // channel control block address register
            set_dma_tx_address.transfer_length.write(core::mem::size_of::<u32>() as u32);
            set_dma_tx_address.next_control_block_address.write(self.tx_control_block_memory.bus_address + (disable_tx_cb_index * core::mem::size_of::<DmaControlBlock>()) as u32);


            disable_dma_tx_address.transfer_information.write(Self::DMA_TI_SRC_INC | Self::DMA_TI_DEST_INC | Self::DMA_TI_WAIT_RESP);
            disable_dma_tx_address.source_address.write(self.dma_constant_memory.bus_address);
            disable_dma_tx_address.destination_address.write(Self::DMA_SPI0_CS_PHYS_ADDRESS);
            disable_dma_tx_address.transfer_length.write(core::mem::size_of::<u32>() as u32);
            disable_dma_tx_address.next_control_block_address.write(self.tx_control_block_memory.bus_address + (start_tx_cb_index * core::mem::size_of::<DmaControlBlock>()) as u32);

    
            start_dma_tx_address.transfer_information.write(Self::DMA_TI_SRC_INC | Self::DMA_TI_DEST_INC | Self::DMA_TI_WAIT_RESP);
            start_dma_tx_address.source_address.write(self.dma_constant_memory.bus_address + (i * core::mem::size_of::<u32>()) as u32);
            start_dma_tx_address.destination_address.write(Self::DMA_DMA0_CS_PHYS_ADDRESS + (Self::TX_CHANNEL_NUMBER as u32 * Self::DMA_CHANNEL_REGISTERS_SIZE as u32));
            start_dma_tx_address.transfer_length.write(core::mem::size_of::<u32>() as u32);
            start_dma_tx_address.next_control_block_address.write(self.rx_control_block_memory.bus_address + (i * core::mem::size_of::<DmaControlBlock>()) as u32);


            rx_control_block = &mut *((self.rx_control_block_memory.virtual_address_ptr as *mut DmaControlBlock).add(i));
            rx_control_block.transfer_information.write(Self::dma_ti_permap(Self::DMA_TI_PERMAP_SPI_RX) | Self::DMA_TI_SRC_DREQ | Self::DMA_TI_DEST_IGNORE);
            rx_control_block.source_address.write(Self::DMA_SPI0_FIFO_PHYS_ADDRESS);
            rx_control_block.destination_address.write(0);
            rx_control_block.transfer_length.write(Self::DMA_SPI_CHUNK_SIZE as u32 - Self::DMA_SPI_HEADER_SIZE);       // without the 4 byte header
            rx_control_block.next_control_block_address.write(0);
        }
    }

    pub fn start_dma_transfer(&mut self, data:&[u8; SPI_BUFFER_SIZE], transfer_active_flag:u8){        
        unsafe{
            if self.tx_dma.control_status.read() & 0x100 != 0{
                log::error!("Error in the tx dma");
            }

            let data_len = Self::DMA_SPI_CHUNK_SIZE - Self::DMA_SPI_HEADER_SIZE as usize;  // Removing the first 4 bytes from this length param
            let header = [transfer_active_flag, 0, (data_len & 0xFF) as u8,  /*making sure this is little endian order*/ (data_len >> 8) as u8];

            let chunks = data.chunks_exact(Self::DMA_SPI_CHUNK_SIZE - Self::DMA_SPI_HEADER_SIZE as usize);
            let mut array:[u8;Self::DMA_SPI_TRANSFER_SIZE] = [0;Self::DMA_SPI_TRANSFER_SIZE];
            let mut i = 0;
            for chunk in chunks{
                core::ptr::copy_nonoverlapping(header.as_ptr(), array.as_mut_ptr().add(i * Self::DMA_SPI_CHUNK_SIZE), 4);
                core::ptr::copy_nonoverlapping(chunk.as_ptr(), array.as_mut_ptr().add(4 + (i * Self::DMA_SPI_CHUNK_SIZE)), Self::DMA_SPI_CHUNK_SIZE - 4);
                i += 1;
            }

            core::ptr::copy_nonoverlapping(array.as_ptr(), self.source_buffer_memory.virtual_address_ptr as *mut u8, array.len());
            
            self.tx_dma.control_block_address.write(self.tx_control_block_memory.bus_address);
            self.rx_dma.control_block_address.write(self.rx_control_block_memory.bus_address);

            memory_barrier();   // Sync all the memory operations happened in this function 
            // Starting the dma transfer
            self.tx_dma.control_status.write(Self::DMA_CS_ACTIVE | Self::DMA_CS_END);
            self.rx_dma.control_status.write(Self::DMA_CS_ACTIVE | Self::DMA_CS_END);
            // Since the DMA controller writes to the SPI registers adding a barrier (even though it wrties afterwards to the DMA registers)
            memory_barrier();   // Change DMA to SPI
        }
    }

    pub fn end_dma_transfer(&self){
        const TIME_TO_ABORT_AS_MICRO:i32 = 1_000_000;
        // Wait for the last trasfer to end
        let mut counter = 0;
        while self.tx_dma.control_status.read() & Self::DMA_CS_ACTIVE != 0 {
            // Self::sleep_us(1);
            counter += 1;
            if counter > TIME_TO_ABORT_AS_MICRO{
                core::panic!("ERROR! tx dma channel is not responding, a reboot is suggested");
            }
        }
        while self.rx_dma.control_status.read() & Self::DMA_CS_ACTIVE != 0 {
            // Self::sleep_us(1);
            counter += 1;
            if counter > TIME_TO_ABORT_AS_MICRO{
                core::panic!("ERROR! rx dma channel is not responding, a reboot is suggested");
            }
        }
    }
}

impl Drop for DmaSpiTransferer{
    fn drop(&mut self) {
        // Finish current dma operation
        self.end_dma_transfer();
        
        // reset the dma channels before releasing the memory
        // reset the dma channels
        self.tx_dma.control_status.write(Self::DMA_CS_RESET);
        self.rx_dma.control_status.write(Self::DMA_CS_RESET);
        // clear the permaps for the channels
        self.tx_dma.transfer_information.write(0);
        self.rx_dma.transfer_information.write(0);
        // disable the channels I used
        let mask = !((1 << Self::TX_CHANNEL_NUMBER) | (1 << Self::RX_CHANNEL_NUMBER));
        self.dma_enable_register.write(self.dma_enable_register.read() & mask);
    }
}