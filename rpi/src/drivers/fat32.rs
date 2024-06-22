use core::mem::size_of;

use crate::peripherals::compile_time_size_assert;
use super::{as_mut_buffer, disk::*};

#[derive(Default)]
#[repr(C, packed)]
struct Fat32BiosParameterBlock{
    // Base fields
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors_count: u16,
    fats_count: u8,
    root_entrires_count:u16,
    total_sectors_count_16: u16,
    media: u8,
    sectors_per_fat_16:u16,
    sectors_per_track:u16,
    heads_count:u16,
    hidden_sectors_count:u32,
    total_sectors_count_32:u32,

    // extended fat32 fields
    sectors_per_fat_32:u32,
    extended_flags:u16,
    fs_version:u16,
    root_dir_first_cluster:u32,
    fs_info_sector:u16,
    backup_boot_sector:u16,
    _reserved0:[u8;12],
}
compile_time_size_assert!(Fat32BiosParameterBlock, 53);

#[repr(C, packed)]
struct Fat32BootSector{
    jump_boot:[u8;3],
    oem_name:[u8;8],
    fat32_bpb:Fat32BiosParameterBlock,
    drive_num:u8,
    _reserved1:u8,
    boot_signature:u8,
    volume_id:u32,
    volume_label:[u8;11],
    fs_type_label:[u8;8],
    _pad:[u8;420],
    signature_word:[u8;2],
}
compile_time_size_assert!(Fat32BootSector, 512);

impl Default for Fat32BootSector{
    fn default() -> Self {
        Self { 
            jump_boot: Default::default(), oem_name: Default::default(), fat32_bpb: Default::default(), drive_num: Default::default(), 
            _reserved1: Default::default(), boot_signature: Default::default(), volume_id: Default::default(),
            volume_label: Default::default(), fs_type_label: Default::default(), _pad: [0;420], signature_word: Default::default() 
        }
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
struct FatShortDirEntry{
    file_name:[u8;8],
    file_extension:[u8;3],
    attributes:u8,
    nt_reserve:u8,
    creation_time_tenth_secs:u8,
    creation_time:u16,
    creation_date:u16,
    last_access_date:u16,
    first_cluster_index_high:u16,
    last_write_time:u16,
    last_write_date:u16,
    first_cluster_index_low:u16,
    file_size:u32,
}

// This struct is for support to the long filenames that I will add later
// unused for now
#[derive(Clone, Copy)]
#[repr(C,packed)]
struct FatLongDirEntry{
    order:u8,
    name1:[u16;5],
    attributes:u8,
    ext_type:u8,
    check_sum:u8,
    name2:[u16;6],
    _first_cluster_low:u16,
    name3:[u16;2]
}

const DISK_PARTITION_INDEX:u8       = 0;
const SECTOR_SIZE:u32               = 512; 
const FAT_ENTRY_SIZE:usize          = size_of::<u32>(); // each fat entry in fat32 is 4 the size of u32
const FAT_ENTRY_EOF_INDEX:u32       = 0x0FFF_FFFF;
const FAT_ENTRY_MASK:u32            = 0x0FFF_FFFF;
const DELETED_DIR_ENTRY_PREFIX:u8   = 0xE5;
const DIR_EOF_PREFIX:u8             = 0;

const FAT_DIR_ENTRIES_PER_SECTOR:usize = SECTOR_SIZE as usize / core::mem::size_of::<FatShortDirEntry>();

const ATTR_READ_ONLY:u8 = 0x1;
const ATTR_HIDDEN:u8    = 0x2;
const ATTR_SYSTEM:u8    = 0x4;
const ATTR_VOLUME_ID:u8 = 0x8;
const ATTR_LONG_NAME:u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID;

#[derive(Default, Clone, Copy)]
pub struct FileEntry{
    name:[u8;Self::FILENAME_SIZE],
    first_cluster_index:u32,
    pub size:u32,
}

impl FileEntry{
    pub const FILENAME_SIZE:usize = 11;

    pub fn get_name<'a>(&'a self)->&'a str{
        core::str::from_utf8(&self.name).unwrap().trim()
    }

    pub fn get_extension<'a>(&'a self)->&'a str{
        core::str::from_utf8(&self.name[8..]).unwrap().trim()
    }
}

pub struct Fat32{
    disk: Disk,
    boot_sector:Fat32BootSector,
    partition_start_sector_index:u32,
}

impl Fat32{
    pub fn new()->Self{
        let mut disk = Disk::new();
        // This driver currently support only a single partition (some has more than one for backup or stuff I dont know)
        let bpb_sector_index = disk.get_partition_first_sector_index(DISK_PARTITION_INDEX);

        let mut boot_sector:Fat32BootSector = Default::default();
        let buffer = unsafe{as_mut_buffer(&mut boot_sector)};
        let _ = disk.read(bpb_sector_index, buffer);

        let fs_type_label = boot_sector.fs_type_label.clone();
        if &fs_type_label[0..3] != b"FAT"{
            core::panic!("File system is not FAT");
        }
        if boot_sector.fat32_bpb.sectors_per_fat_16 != 0{
            core::panic!("Detected FAT16 and not FAT32 file system");
        }
        let bytes_per_sector = boot_sector.fat32_bpb.bytes_per_sector as u32;
        if bytes_per_sector != disk.get_block_size() || bytes_per_sector != SECTOR_SIZE{
            core::panic!("Currently dont support fat32 disks with sectors size other than {}", SECTOR_SIZE);
        }
        let fat_count = boot_sector.fat32_bpb.fats_count;
        log::debug!("FAT count: {}", fat_count);

        return Self { disk, boot_sector, partition_start_sector_index:bpb_sector_index };
    }

    pub fn root_dir_list<const RESULT_MAX_LEN:usize>(&mut self, offset:usize)->[Option<FileEntry>;RESULT_MAX_LEN]{
        let root_start_sector_index = self.get_cluster_start_sector_index(self.boot_sector.fat32_bpb.root_dir_first_cluster);
        
        let mut root_dir_files_count = 0;
        let mut output_dir = [None;RESULT_MAX_LEN];
        let mut sector_offset = 0;
        let mut discard = offset;

        'search: loop{
            let mut root_dir = [FatShortDirEntry::default();FAT_DIR_ENTRIES_PER_SECTOR];
            let buffer = unsafe{as_mut_buffer(&mut root_dir)};
            sector_offset += self.disk.read(root_start_sector_index + sector_offset, buffer);
            for dir in root_dir{
                if dir.file_name[0] == DIR_EOF_PREFIX{
                    break 'search;
                }
                if dir.file_name[0] == DELETED_DIR_ENTRY_PREFIX{
                    continue;
                }
                if dir.attributes == ATTR_LONG_NAME{
                    continue;
                    // handle long file names here
                }
                if discard > 0{
                    discard -= 1;
                    continue;
                }

                let mut filename:[u8;11] = [0;11];
                filename[..8].copy_from_slice(&dir.file_name);
                filename[8..11].copy_from_slice(&dir.file_extension);
                let first_cluster_index = dir.first_cluster_index_low as u32 | ((dir.first_cluster_index_high as u32) << 16);
                
                output_dir[root_dir_files_count] = Some(FileEntry{ name: filename, first_cluster_index, size: dir.file_size });
                root_dir_files_count += 1;
                if root_dir_files_count == RESULT_MAX_LEN{
                    break 'search;
                }
            }
        }

        return output_dir;
    }

    /// Reads a file from the first FAT
    pub fn read_file(&mut self, file_entry:&FileEntry, output:&mut [u8]){
        log::debug!("Reading file {}, size {}, cluster: {}", file_entry.get_name(), file_entry.size, file_entry.first_cluster_index);
        let fat_offset = file_entry.first_cluster_index * FAT_ENTRY_SIZE as u32;      
        let mut fat_sector_number = self.partition_start_sector_index + self.boot_sector.fat32_bpb.reserved_sectors_count as u32 + (fat_offset / SECTOR_SIZE);
        let mut fat_entry_offset = (fat_offset % SECTOR_SIZE) as usize;
        
        let sectors_per_cluster = self.boot_sector.fat32_bpb.sectors_per_cluster;
        let mut current_cluster = file_entry.first_cluster_index;
        let mut cluster_counter = 0;
        let mut fat_buffer = [0; SECTOR_SIZE as usize];
        let _ = self.disk.read(fat_sector_number, &mut fat_buffer);

        loop{
            let start_sector = self.get_cluster_start_sector_index(current_cluster);
            let start_index = sectors_per_cluster as usize * cluster_counter * SECTOR_SIZE as usize;
            let end_index = start_index + (sectors_per_cluster as usize * SECTOR_SIZE as usize);
            let _ = self.disk.read(start_sector, &mut output[start_index..end_index]);
            
            let fat_entry = u32::from_ne_bytes(fat_buffer[fat_entry_offset .. fat_entry_offset + FAT_ENTRY_SIZE].try_into().unwrap()) & FAT_ENTRY_MASK; 
            if fat_entry == FAT_ENTRY_EOF_INDEX{
                return;
            }
            current_cluster = fat_entry;
            cluster_counter += 1;
            fat_entry_offset += FAT_ENTRY_SIZE;
            if fat_entry_offset >= SECTOR_SIZE as usize{
                fat_entry_offset = 0;
                fat_sector_number += 1;
                let _ = self.disk.read(fat_sector_number, &mut fat_buffer);
            }
        }

    }

    fn get_cluster_start_sector_index(&self, cluster:u32)->u32{
        const FIRST_DATA_CLUSTER:u32 = 2;

        self.partition_start_sector_index + self.boot_sector.fat32_bpb.reserved_sectors_count as u32 + 
        ((cluster - FIRST_DATA_CLUSTER) * self.boot_sector.fat32_bpb.sectors_per_cluster as u32) + 
        (self.boot_sector.fat32_bpb.sectors_per_fat_32 * self.boot_sector.fat32_bpb.fats_count as u32)
    }
}