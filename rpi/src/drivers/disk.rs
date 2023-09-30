use crate::peripherals::{Emmc, PERIPHERALS, compile_time_size_assert};
use super::as_mut_buffer;

#[repr(C, packed)]
struct PartitionEntry{
    status:u8,
    first_sector_chs_address:[u8;3],
    partition_type:u8,
    last_sector_chs_address:[u8;3],
    first_sector_index:u32,
    sectors_count:u32,
}

impl Default for PartitionEntry{
    fn default() -> Self {
        Self { status: Default::default(), first_sector_chs_address: Default::default(), partition_type: Default::default(), last_sector_chs_address: Default::default(), first_sector_index: Default::default(), sectors_count: Default::default() }
    }
}

#[repr(C)]
struct MasterBootRecord{
    boot_code:[u8;446],
    partitions:[PartitionEntry;4],
    boot_signature:u16,
}
compile_time_size_assert!(MasterBootRecord, 512);

impl Default for MasterBootRecord{
    fn default() -> Self {
        Self { boot_code: [0;446], partitions: Default::default(), boot_signature: Default::default() }
    }
}

pub struct Disk{
    emmc:Emmc,
    mbr:MasterBootRecord
}

impl Disk{
    const BLOCK_SIZE:u32 = Emmc::get_block_size();

    pub fn new()->Self{
        let mut emmc = unsafe{PERIPHERALS.take_emmc()};
        emmc.init();

        let mut mbr = MasterBootRecord::default();
        let buffer = unsafe{as_mut_buffer(&mut mbr)};
        
        if !emmc.read(buffer){
            core::panic!("Cant read MBR from disk");
        }
        if mbr.boot_signature != 0xAA55{
            core::panic!("Bad boot signature in disk: {:#X}", mbr.boot_signature);
        }

        Self { emmc, mbr }
    }

    pub fn read(&mut self, block_index:u32, buffer:&mut [u8]){
        let buffer_len_reminder = buffer.len() % Self::BLOCK_SIZE as usize;
        let max_aligned_buffer_len = buffer.len() - buffer_len_reminder;
        let aligned_buffer = &mut buffer[..max_aligned_buffer_len];

        self.emmc.seek((block_index * Self::BLOCK_SIZE) as u64);
        // Verify the buffer is larger than a single block 
        if aligned_buffer.len() != 0{
            self.emmc_read(aligned_buffer);
            // early return if the buffer is aligned
            if buffer_len_reminder == 0 {return};
        }
        // handle the case buffer length is not aligned for block size
        let mut temp_buffer:[u8;Self::BLOCK_SIZE as usize] = [0;Self::BLOCK_SIZE as usize];
        self.emmc.seek(((block_index + (max_aligned_buffer_len as u32 / Self::BLOCK_SIZE)) * Self::BLOCK_SIZE) as u64);
        self.emmc_read(&mut temp_buffer);
        buffer[max_aligned_buffer_len..].copy_from_slice(&mut temp_buffer[..buffer_len_reminder]);
    }

    fn emmc_read(&mut self, buffer: &mut [u8]) {
        if !self.emmc.read(buffer){
            core::panic!("Error while reading object of size: {}", buffer.len());
        }
    }

    pub fn write(&mut self, block_index:u32, buffer:&[u8]){
        let buffer_len_reminder = buffer.len() % Self::BLOCK_SIZE as usize;
        let max_aligned_buffer_len = buffer.len() - buffer_len_reminder;
        let aligned_buffer = &buffer[..max_aligned_buffer_len];

        self.emmc.seek((block_index * Self::BLOCK_SIZE) as u64);
        if aligned_buffer.len() != 0{
            self.emmc_write(aligned_buffer);
            // early return since the buffer is aligned
            if buffer_len_reminder == 0 {return};
        }
        // handle the case buffer length is not aligned for block size
        let mut temp_buffer:[u8;Self::BLOCK_SIZE as usize] = [0;Self::BLOCK_SIZE as usize];
        temp_buffer[max_aligned_buffer_len..].copy_from_slice(&buffer[..buffer_len_reminder]);
        self.emmc.seek(((block_index + (max_aligned_buffer_len as u32 / Self::BLOCK_SIZE)) * Self::BLOCK_SIZE) as u64);
        self.emmc_write(&temp_buffer);
    }

    fn emmc_write(&mut self, buffer: &[u8]) {
        if !self.emmc.write(buffer){
            core::panic!("Error while writing object of size: {}", buffer.len());
        }
    }

    pub fn get_partition_first_sector_index(&self, partition_index:u8)->u32{
        self.mbr.partitions[partition_index as usize].first_sector_index
    }

    pub const fn get_block_size()->u32{Self::BLOCK_SIZE}
}