use core::mem::size_of;

use arrayvec::ArrayVec;

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

impl FatShortDirEntry{
    pub fn get_first_cluster_index(&self)->u32{
        self.first_cluster_index_low as u32 | ((self.first_cluster_index_high as u32) << 16)
    }
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

struct FatIndex{
    sector_number:u32,
    sector_offset:usize,
}

impl FatIndex{
    fn get_fat_entry(&mut self, buffer:&[u8;FAT_BUFFER_SIZE])->u32{
       let result = u32::from_ne_bytes(buffer[self.sector_offset .. self.sector_offset + FAT_ENTRY_SIZE].try_into().unwrap()) & FAT_ENTRY_MASK;
       self.sector_offset += FAT_ENTRY_SIZE;
       return result;
    }
}

#[derive(Clone)]
struct FatSegment{
    value:u32,
    len:u32,
}

// Currently the driver support only 0x100 files in the root directory
const MAX_FILES: usize = 0x100;
// Assuming each files is 0x100 clusters in average
const MAX_FAT_SEGMENTS_COUNT: usize = MAX_FILES * 0x100;

const FAT_BUFFER_SIZE:usize = SECTOR_SIZE as usize * 100;

pub struct Fat32{
    disk: Disk,
    boot_sector:Fat32BootSector,
    partition_start_sector_index:u32,

    clusters_count:u32,
    fat_table_cache: ArrayVec<FatSegment, MAX_FAT_SEGMENTS_COUNT>,
    root_dir_cache: ArrayVec<FatShortDirEntry, MAX_FILES>,
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

        let fat32_data_sectors = boot_sector.fat32_bpb.total_sectors_count_32 - (boot_sector.fat32_bpb.reserved_sectors_count as u32 + (boot_sector.fat32_bpb.sectors_per_fat_32 as u32 * boot_sector.fat32_bpb.fats_count as u32));
        let clusters_count = fat32_data_sectors / boot_sector.fat32_bpb.sectors_per_cluster as u32;

        let mut fat32 = Self { disk, boot_sector, partition_start_sector_index:bpb_sector_index, clusters_count,
            fat_table_cache: ArrayVec::<FatSegment, MAX_FAT_SEGMENTS_COUNT>::new(),
            root_dir_cache: ArrayVec::<FatShortDirEntry, MAX_FILES>::new()
        };
        fat32.init_root_directory_cache();
        fat32.init_fat_table_cache();

        return fat32;
    }

    fn init_root_directory_cache(&mut self){
        let root_start_sector_index = self.get_cluster_start_sector_index(self.boot_sector.fat32_bpb.root_dir_first_cluster);
        let mut sector_offset = 0;
        'search: loop{
            let mut root_dir = [FatShortDirEntry::default();FAT_DIR_ENTRIES_PER_SECTOR];
            let buffer = unsafe{as_mut_buffer(&mut root_dir)};
            sector_offset += self.disk.read(root_start_sector_index + sector_offset, buffer);
            for dir in root_dir{
                if dir.file_name[0] == DIR_EOF_PREFIX {
                    break 'search;
                }
                self.root_dir_cache.push(dir);
            }
        }
    }

    fn init_fat_table_cache(&mut self){
        let mut fat_index = FatIndex{sector_number: self.get_fat_start_sector(), sector_offset: 0};
        let mut fat_buffer = [0; FAT_BUFFER_SIZE];
        let _ = self.disk.read(fat_index.sector_number, &mut fat_buffer);

        // The fat has entry per cluster in the volume, were adding 2 for the first 2 reserved entries (0,1)
        // This way the array is larger by 2 (fat entry at position clusters_count + 1 is the last valid entry)
        let fat_entries_count = self.clusters_count + 1;
        log::debug!("fat entries count {}", fat_entries_count);
        let mut current_segment = FatSegment{value: self.get_fat_entry(&mut fat_index, &mut fat_buffer), len: 1};
        for _ in 0..=fat_entries_count{
            let fat_entry = self.get_fat_entry(&mut fat_index, &mut fat_buffer);
            if fat_entry == current_segment.value{
                current_segment.len += 1;
                continue;
            }
            self.fat_table_cache.push(current_segment.clone());
            current_segment = FatSegment{value: fat_entry, len: 1};
        }
        self.fat_table_cache.push(current_segment);
        log::debug!("Fat segments count {}", self.fat_table_cache.len());
    }

    pub fn root_dir_list<const RESULT_MAX_LEN:usize>(&mut self, offset:usize)->ArrayVec<FileEntry, RESULT_MAX_LEN>{
        let mut output_dir = ArrayVec::<FileEntry,RESULT_MAX_LEN>::new();
        let mut discard = offset;

        for dir in &self.root_dir_cache{
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
            let first_cluster_index = dir.get_first_cluster_index();
            
            output_dir.push(FileEntry{ name: filename, first_cluster_index, size: dir.file_size });
            if output_dir.remaining_capacity() == 0{
                break;
            }
        }

        return output_dir;
    }

    /// Reads a file from the first FAT
    pub fn read_file(&mut self, file_entry:&FileEntry, output:&mut [u8]){
        log::debug!("Reading file {}, size {}, cluster: {}", file_entry.get_name(), file_entry.size, file_entry.first_cluster_index);
        let mut fat_index: FatIndex = self.get_fat_index(file_entry.first_cluster_index);
        
        let sectors_per_cluster = self.boot_sector.fat32_bpb.sectors_per_cluster;
        let mut current_cluster = file_entry.first_cluster_index;
        let mut cluster_counter = 0;
        let mut fat_buffer = [0; FAT_BUFFER_SIZE];
        let _ = self.disk.read(fat_index.sector_number, &mut fat_buffer);

        loop{
            let start_sector = self.get_cluster_start_sector_index(current_cluster);
            let start_index = sectors_per_cluster as usize * cluster_counter * SECTOR_SIZE as usize;
            let end_index = start_index + (sectors_per_cluster as usize * SECTOR_SIZE as usize);
            let _ = self.disk.read(start_sector, &mut output[start_index..end_index]);
            
            let fat_entry =  self.get_fat_entry(&mut fat_index, &mut fat_buffer);
            if fat_entry == FAT_ENTRY_EOF_INDEX{
                return;
            }
            current_cluster = fat_entry;
            cluster_counter += 1;
        }
    }

    /// Write a file to the root dir
    pub fn write_file(&mut self, filename:&str, content:&mut [u8]){
        let free_fat_entry = self.get_free_fat_entry(1000).expect("Filesystem is too large, cant find free entry");
        
    }

    fn get_fat_entry(&mut self, fat_index:&mut FatIndex, fat_buffer:&mut [u8; FAT_BUFFER_SIZE])->u32{
        let fat_entry = fat_index.get_fat_entry(&fat_buffer);
        if fat_index.sector_offset >= FAT_BUFFER_SIZE{
            fat_index.sector_offset = 0;
            fat_index.sector_number += 1;
            let _ = self.disk.read(fat_index.sector_number, fat_buffer);
        }
        return fat_entry;
    }

    fn get_fat_index(&self, first_cluster_index:u32)->FatIndex{
        let fat_offset = first_cluster_index * FAT_ENTRY_SIZE as u32;
        return FatIndex { 
            sector_number:self.get_fat_start_sector() + (fat_offset / SECTOR_SIZE),
            sector_offset: (fat_offset % SECTOR_SIZE) as usize
        };
    }

    fn get_fat_start_sector(&self) -> u32 {
        self.partition_start_sector_index + self.boot_sector.fat32_bpb.reserved_sectors_count as u32
    }

    fn get_free_fat_entry(&mut self, max_iterations:u32) -> Option<u32> {
        let root_start_sector_index = self.get_cluster_start_sector_index(self.boot_sector.fat32_bpb.root_dir_first_cluster);
        let mut sector_offset = 0;
        let mut iteration_counter = 0;
        let mut result = None;
        'search_loop: loop{    
            let mut root_dir = [FatShortDirEntry::default();FAT_DIR_ENTRIES_PER_SECTOR];
            let buffer = unsafe{as_mut_buffer(&mut root_dir)};
            sector_offset += self.disk.read(root_start_sector_index + sector_offset, buffer);
            for dir in root_dir{
                let dir_prefix = dir.file_name[0];
                if dir_prefix == DELETED_DIR_ENTRY_PREFIX {
                    result = result.or(Some(iteration_counter));
                }
                else if dir_prefix == DIR_EOF_PREFIX {
                    return result.or(Some(iteration_counter));
                }
                else{
                    let mut fat_index = self.get_fat_index(dir.get_first_cluster_index());
                    let mut fat_buffer = [0;FAT_BUFFER_SIZE];
                    let _ = self.disk.read(fat_index.sector_number, &mut fat_buffer);
                    let mut fat_entry = self.get_fat_entry(&mut fat_index, &mut fat_buffer);
                    while fat_entry != FAT_ENTRY_EOF_INDEX{
                        // self.set_occupied_cluser(fat_entry);
                        fat_entry = self.get_fat_entry(&mut fat_index, &mut fat_buffer);
                    }
                }
                iteration_counter += 1;
                if iteration_counter == max_iterations{
                    break 'search_loop;
                }
            }
        }
        return result;
    }

    fn get_cluster_start_sector_index(&self, cluster:u32)->u32{
        const FIRST_DATA_CLUSTER:u32 = 2;

        self.partition_start_sector_index + self.boot_sector.fat32_bpb.reserved_sectors_count as u32 + 
        ((cluster - FIRST_DATA_CLUSTER) * self.boot_sector.fat32_bpb.sectors_per_cluster as u32) + 
        (self.boot_sector.fat32_bpb.sectors_per_fat_32 * self.boot_sector.fat32_bpb.fats_count as u32)
    }
}