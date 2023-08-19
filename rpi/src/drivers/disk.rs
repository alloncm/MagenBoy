use crate::peripherals::{Emmc, PERIPHERALS, compile_time_size_assert};
use super::as_mut_buffer;

#[repr(C, packed)]
struct PartitionEntry{
    status:u8,
    frist_sector_chs_address:[u8;3],
    partition_type:u8,
    last_sector_chs_address:[u8;3],
    first_sector_index:u32,
    sectors_count:u32,
}

impl Default for PartitionEntry{
    fn default() -> Self {
        Self { status: Default::default(), frist_sector_chs_address: Default::default(), partition_type: Default::default(), last_sector_chs_address: Default::default(), first_sector_index: Default::default(), sectors_count: Default::default() }
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

    /// Returns the number of blocks the read operation fetched
    /// The user knows how much of the buffer is filled
    pub fn read(&mut self, block_index:u32, buffer:&mut [u8]) -> u32 {
        let block_size = self.emmc.get_block_size();
        let buffer_size = buffer.len();
        if buffer_size  % block_size as usize != 0{
            core::panic!("buffer size must be a division of block size: {}", block_size);
        }
        self.emmc.seek((block_index * block_size) as u64);
        if !self.emmc.read(buffer){
            core::panic!("Error while reading object of size: {}", buffer_size);
        }
        return  buffer_size as u32 / block_size;
    }

    pub fn get_partition_first_sector_index(&self, partition_index:u8)->u32{
        self.mbr.partitions[partition_index as usize].first_sector_index
    }

    pub fn get_block_size(&self)->u32{self.emmc.get_block_size()}
}