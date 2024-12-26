use crate::drivers::disk::Disk;

use super::SECTOR_SIZE;

const FAT_ENTRY_SIZE:usize  = 4;
const FAT_ENTRY_MASK:u32    = 0x0FFF_FFFF;

#[derive(Clone, Debug)]
pub struct FatIndex{
    sector_index:u32,
    sector_offset:usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FatSegmentState{
    Free,
    Allocated,
    AllocatedEof,
    Reserved,
    Bad,
}

impl From<u32> for FatSegmentState{
    fn from(value: u32) -> Self {
        match value{
            0 => Self::Free,
            2..=0xFFF_FFF5 => Self::Allocated,
            0xFFF_FFFF => Self::AllocatedEof,
            0xFFF_FFF7 => Self::Bad,
            _ => Self::Reserved
        }
    }
}

impl FatSegmentState{
    /// Checks whether a value should be part of this segment or not
    pub fn should_continue_segment(&self, other: &Self)->bool{
        // AllocatedEof is should never continue segment 
        // otherwise fallback to check raw values of the enum
        if *self == Self::AllocatedEof || *other == Self::AllocatedEof{
            return false;
        }
        return self == other;
    }
}

#[derive(Clone, Copy)]
pub struct FatInfo{
    first_fat_start_sector:usize,
    sectors_per_fat:usize,
    fats_count:usize
}

impl FatInfo{
    pub fn new(first_fat_start_sector:usize, sectors_per_fat:usize, fats_count: usize)->Self{
        Self { first_fat_start_sector, sectors_per_fat, fats_count }
    }
}

// This is the default size of a fat buffer
// the actual size is just tweaking between fewer read operation and smaller buffer
pub const FAT_BUFFER_SIZE:usize = SECTOR_SIZE as usize * 100;

pub struct FatBuffer<const FBS:usize = FAT_BUFFER_SIZE>{
    buffer:[u8;FBS],
    buffer_len: usize,
    fat_start_index:FatIndex,
    fat_internal_index:FatIndex,
    fat_info:FatInfo,
}

impl<const FBS:usize> FatBuffer<FBS>{
    // The buffer Im reading will be the same buffer that Im writing back
    // so it must be aligned in order to not corrupt the fat table
    pub fn new(fat_info:FatInfo, first_cluster_index:usize, entries_count: Option<usize>, disk: &mut Disk)->Self{
        let entries_count = entries_count.unwrap_or((FBS - SECTOR_SIZE) / FAT_ENTRY_SIZE);      // The max size is smaller cause I need some padding space for alignment
        let mut buffer = [0; FBS];
        let fat_offset = first_cluster_index * FAT_ENTRY_SIZE;
        let fat_index = FatIndex{ sector_index: (fat_info.first_fat_start_sector + (fat_offset / SECTOR_SIZE)) as u32, sector_offset: fat_offset % SECTOR_SIZE };
        
        // Align the end read to SECTOR_SIZE, since the FAT table is not aligned we need to read exactly X sectors in order to be able to write them back later
        let fat_end_read = (entries_count * FAT_ENTRY_SIZE) + (SECTOR_SIZE - ((entries_count * FAT_ENTRY_SIZE) % SECTOR_SIZE));
        if fat_end_read > FBS{
            core::panic!("Error fat entries count is too much: expected:{}, actual: {}", FBS / FAT_ENTRY_SIZE, entries_count);
        }
        disk.read(fat_index.sector_index, &mut buffer[..fat_end_read]);
        return Self { buffer, fat_start_index: fat_index.clone(), fat_internal_index: fat_index, buffer_len: fat_end_read, fat_info };
    }

    /// On success returns the FAT entry, on error returns the last valid fat index
    pub fn read(&mut self)->Result<u32, FatIndex>{
        let internal_sector_index = self.get_internal_sector_index()?;
        let start_index = (internal_sector_index * SECTOR_SIZE) + self.fat_internal_index.sector_offset;
        let end_index = start_index + FAT_ENTRY_SIZE;
        let entry = Self::bytes_to_fat_entry(self.buffer[start_index .. end_index].try_into().unwrap());
        self.fat_internal_index.sector_offset += FAT_ENTRY_SIZE;
        if self.fat_internal_index.sector_offset >= SECTOR_SIZE{
            self.fat_internal_index.sector_index += 1;
            self.fat_internal_index.sector_offset = 0;
        }
        // Mask the entry to hide the reserved bits
        return Ok(entry & FAT_ENTRY_MASK);
    }

    /// On error returns the last valid fat index
    pub fn write(&mut self, mut value:u32)->Result<(), FatIndex>{
        let internal_sector_index = self.get_internal_sector_index()?;
        let start_index = (internal_sector_index * SECTOR_SIZE) + self.fat_internal_index.sector_offset;
        let end_index = start_index + FAT_ENTRY_SIZE;
        let entry = Self::bytes_to_fat_entry(self.buffer[start_index .. end_index].try_into().unwrap());
        let reserved_bits = entry & (!FAT_ENTRY_MASK);
        value = (value & FAT_ENTRY_MASK) | reserved_bits;
        self.buffer[start_index ..  end_index].copy_from_slice(&Self::fat_entry_to_bytes(value));
        self.fat_internal_index.sector_offset += FAT_ENTRY_SIZE;
        if self.fat_internal_index.sector_offset >= SECTOR_SIZE{
            self.fat_internal_index.sector_index += 1;
            self.fat_internal_index.sector_offset = 0;
        }
        return Ok(());
    }

    /// Sync the fat buffer to the disk
    pub fn flush(&mut self, disk:&mut Disk){
        // Sync all the fat sectors to disk
        for i in 0..self.fat_info.fats_count{
            let start_sector = self.fat_start_index.sector_index + (self.fat_info.sectors_per_fat * i) as u32;
            let _ = disk.write(start_sector, &mut self.buffer[..self.buffer_len]);
        }
    }

    fn get_internal_sector_index(&self)->Result<usize, FatIndex>{
        let internal_sector_index = (self.fat_internal_index.sector_index - self.fat_start_index.sector_index) as usize;
        if internal_sector_index * SECTOR_SIZE >= self.buffer_len{
            return Err(self.fat_internal_index.clone());
        }
        return Ok(internal_sector_index);
    }

    fn bytes_to_fat_entry(buffer:&[u8;FAT_ENTRY_SIZE])->u32 {u32::from_ne_bytes(*buffer)}
    fn fat_entry_to_bytes(entry:u32)->[u8;FAT_ENTRY_SIZE] {u32::to_ne_bytes(entry)}
}