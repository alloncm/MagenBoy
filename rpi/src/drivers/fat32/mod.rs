mod fat_buffer;

use core::{mem::size_of, ops::ControlFlow};

use arrayvec::ArrayVec;

use crate::peripherals::compile_time_size_assert;
use super::{as_mut_buffer, disk::*};
use fat_buffer::*;

#[derive(Default)]
#[repr(C, packed)]
struct Fat32BiosParameterBlock{
    // Base fields
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors_count: u16,
    fats_count: u8,
    root_entries_count:u16,
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
    file_name:[u8;Self::FILE_NAME_SIZE],
    file_extension:[u8;Self::FILE_EXTENSION_SIZE],
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
    size:u32,
}
compile_time_size_assert!(FatShortDirEntry, 32);

impl FatShortDirEntry{
    const FILE_NAME_SIZE:usize = 8;
    const FILE_EXTENSION_SIZE:usize = 3;
    const FULL_FILENAME_SIZE:usize = Self::FILE_NAME_SIZE + Self::FILE_EXTENSION_SIZE;

    fn new(name:[u8;Self::FILE_NAME_SIZE], extension:[u8;Self::FILE_EXTENSION_SIZE], size:u32)->Self{
        Self { 
            file_name: name, file_extension: extension, attributes: 0, nt_reserve: 0, creation_time_tenth_secs: 0, creation_time: 0, 
            creation_date: 0, last_access_date: 0, first_cluster_index_high:0, last_write_time: 0, last_write_date: 0, first_cluster_index_low:0, size
        }
    }
    fn get_first_cluster_index(&self)->u32{
        self.first_cluster_index_low as u32 | ((self.first_cluster_index_high as u32) << 16)
    }
    fn set_first_cluster_index(&mut self, first_cluster_index:u32){
        self.first_cluster_index_low = (first_cluster_index & 0xFFFF) as u16;
        self.first_cluster_index_high = (first_cluster_index >> 16) as u16;
    }
    fn get_filename(&self)->[u8;Self::FULL_FILENAME_SIZE]{
        let mut filename = [0;Self::FULL_FILENAME_SIZE];
        filename[.. Self::FILE_NAME_SIZE].copy_from_slice(&self.file_name);
        filename[Self::FILE_NAME_SIZE ..].copy_from_slice(&self.file_extension);
        return filename;
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
const SECTOR_SIZE:usize             = Disk::BLOCK_SIZE as usize;
const FAT_ENTRY_EOF_INDEX:u32       = 0x0FFF_FFFF;
const FAT_ENTRY_FREE_INDEX:u32      = 0;
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

#[derive(Clone, Debug)]
pub struct FatSegment{
    pub state:FatSegmentState,
    pub len:u32,
    pub start_index:u32,
}

impl FatSegment{
    pub fn new(value:u32, start_index:u32)->Self{
        Self { state: value.into(), len: 1, start_index}
    }
}

// Currently the driver support only 0x100 files in the root directory
const MAX_FILES: usize = 0x100;
const MAX_FAT_SEGMENTS_COUNT: usize = MAX_FILES * 100;

pub struct Fat32Fs{
    disk: Disk,
    boot_sector:Fat32BootSector,
    partition_start_sector_index:u32,

    clusters_count:u32,
    fat_info:FatInfo,
    fat_table_cache: ArrayVec<FatSegment, MAX_FAT_SEGMENTS_COUNT>,
    root_dir_cache: ArrayVec<FatShortDirEntry, MAX_FILES>,
    root_dir_allocated_clusters_count: u32,
}

impl Fat32Fs{
    pub fn new()->Self{
        let mut disk = Disk::new();
        // This driver currently support only a single partition (some has more than one for backup or stuff I don't know)
        let bpb_sector_index = disk.get_partition_first_sector_index(DISK_PARTITION_INDEX);

        let mut boot_sector:Fat32BootSector = Default::default();
        // SAFETY: Fat32BootSector is repr(C) and therefore safe to transmute to byte slice
        let buffer = unsafe{as_mut_buffer(&mut boot_sector)};
        disk.read(bpb_sector_index, buffer);

        let fs_type_label = boot_sector.fs_type_label.clone();
        if &fs_type_label[0..3] != b"FAT"{
            core::panic!("File system is not FAT");
        }
        if boot_sector.fat32_bpb.sectors_per_fat_16 != 0{
            core::panic!("Detected FAT16 and not FAT32 file system");
        }
        let bytes_per_sector = boot_sector.fat32_bpb.bytes_per_sector as u32;
        if bytes_per_sector != SECTOR_SIZE as u32{
            core::panic!("Currently not supporting fat32 disks with sectors size other than {}", SECTOR_SIZE);
        }
        let fat_count = boot_sector.fat32_bpb.fats_count;
        log::debug!("FAT count: {}", fat_count);

        let fat32_data_sectors = boot_sector.fat32_bpb.total_sectors_count_32 - (boot_sector.fat32_bpb.reserved_sectors_count as u32 + (boot_sector.fat32_bpb.sectors_per_fat_32 as u32 * boot_sector.fat32_bpb.fats_count as u32));
        let clusters_count = fat32_data_sectors / boot_sector.fat32_bpb.sectors_per_cluster as u32;
        let fat_start_sector = (bpb_sector_index + boot_sector.fat32_bpb.reserved_sectors_count as u32) as usize;
        let mut fat32 = Self { 
            fat_info:FatInfo::new( fat_start_sector, boot_sector.fat32_bpb.sectors_per_fat_32 as usize, boot_sector.fat32_bpb.fats_count as usize ),
            disk, boot_sector, partition_start_sector_index:bpb_sector_index, clusters_count,
            fat_table_cache: ArrayVec::<FatSegment, MAX_FAT_SEGMENTS_COUNT>::new(),
            root_dir_cache: ArrayVec::<FatShortDirEntry, MAX_FILES>::new(),
            root_dir_allocated_clusters_count: 0
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
            // SAFETY: FatShortDirEntry is repr(C) and packed and since arrays has the same alingnment as T it is safe
            let buffer = unsafe{as_mut_buffer(&mut root_dir)};
            self.disk.read(root_start_sector_index + sector_offset, buffer);
            sector_offset += 1;     // Since root_dir buffer contains enough entries for exactly 1 sector
            for dir in root_dir{
                // Pushing also the DIR_EOF in order to support syncing the whole dir easily
                self.root_dir_cache.push(dir);
                if dir.file_name[0] == DIR_EOF_PREFIX {
                    break 'search;
                }
            }
        }
        let sectors_per_cluster = self.boot_sector.fat32_bpb.sectors_per_cluster as u32;
        self.root_dir_allocated_clusters_count = sector_offset / sectors_per_cluster + ((sector_offset % sectors_per_cluster) != 0) as u32;
        log::debug!("Root dir allocated clusters count: {}", self.root_dir_allocated_clusters_count);
    }

    // Failed Optimization Attempt: I tried to read the files from the root dir, and once I have all the entries abort and mark the rest of the clusters as free
    // for some reason there were allocated entries on the FAT that I couldn't understand what allocated them and couldn't predict and calculate the expected entries count
    // Ill live it like that for now
    fn init_fat_table_cache(&mut self){
        // This buffer is bigger then the default in order to minimize the number of read operations
        // The value is tweaked for faster reads
        const INIT_FAT_BUFFER_SIZE:usize = FAT_BUFFER_SIZE * 10;
        let mut fat_buffer:FatBuffer<INIT_FAT_BUFFER_SIZE> = FatBuffer::new(self.fat_info, 0, None, &mut self.disk);

        // The fat has entry per cluster in the volume, were adding 2 for the first 2 reserved entries (0,1)
        // This way the array is larger by 2 (fat entry at position clusters_count + 1 is the last valid entry)
        let fat_entries_count = self.clusters_count + 1;
        log::debug!("fat entries count {}", fat_entries_count);

        // Since indices [0,1] are reserved ignore and discard them
        let _ = fat_buffer.read().ok().unwrap();
        let _ = fat_buffer.read().ok().unwrap();

        // Index 2 to bootstrap
        let fat_entry = fat_buffer.read().ok().unwrap();
        let mut current_segment = FatSegment::new(fat_entry, 2);

        for i in 3..=fat_entries_count{
            let fat_entry = fat_buffer.read().unwrap_or_else(|_|{ 
                fat_buffer = FatBuffer::new(self.fat_info, i as usize, None, &mut self.disk);
                fat_buffer.read().ok().unwrap()
            });
            if current_segment.state.should_continue_segment(&FatSegmentState::from(fat_entry)) {
                current_segment.len += 1;
                continue;
            }
            self.fat_table_cache.push(current_segment.clone());
            current_segment = FatSegment::new(fat_entry, i);
        }
        self.fat_table_cache.push(current_segment);
        log::debug!("Fat segments count {}", self.fat_table_cache.len());
    }

    pub fn root_dir_list<const RESULT_MAX_LEN:usize>(&mut self, offset:usize)->ArrayVec<FileEntry, RESULT_MAX_LEN>{
        let mut output_dir = ArrayVec::<FileEntry,RESULT_MAX_LEN>::new();
        let mut discard = offset;

        for dir in &self.root_dir_cache {
            if dir.file_name[0] == DIR_EOF_PREFIX{
                break;
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
            let filename = dir.get_filename();
            let first_cluster_index = dir.get_first_cluster_index();
            
            output_dir.push(FileEntry{ name: filename, first_cluster_index, size: dir.size });
            if output_dir.remaining_capacity() == 0{
                break;
            }
        }

        return output_dir;
    }

    // In this implementation Im trying to read as much many clusters as possible at a time
    // in order to improve performance
    /// Reads a file from the first FAT
    pub fn read_file(&mut self, file_entry:&FileEntry, output:&mut [u8]){
        log::debug!("Reading file {}, size {}, cluster: {}", file_entry.get_name(), file_entry.size, file_entry.first_cluster_index);
        if file_entry.size == 0 {return}

        let sectors_per_cluster = self.boot_sector.fat32_bpb.sectors_per_cluster;
        let mut fat_first_entry = self.fat_table_cache.as_slice().into_iter().find(|t|t.start_index == file_entry.first_cluster_index).unwrap().clone();
        match fat_first_entry.state {
            FatSegmentState::Allocated => fat_first_entry.len += 1,
            FatSegmentState::AllocatedEof => {},
            _ => core::panic!("FAT32 Driver Error! - tried to read file: {} but the fat segment was not allocated: {}", file_entry.get_name(), file_entry.first_cluster_index),
        }
        let mut fat_buffer:FatBuffer = FatBuffer::new(self.fat_info, file_entry.first_cluster_index as usize, Some(fat_first_entry.len as usize), &mut self.disk);

        let mut current_cluster = file_entry.first_cluster_index;
        let mut next_read_cluster = current_cluster;
        let mut clusters_sequence = 1;
        let mut cluster_counter:usize = 0;
        while cluster_counter < fat_first_entry.len as usize{
            let fat_entry = fat_buffer.read().ok().unwrap();
            if current_cluster + 1 == fat_entry{
                current_cluster = fat_entry;
                clusters_sequence += 1;
                continue;
            }
            let start_sector = self.get_cluster_start_sector_index(next_read_cluster);
            let start_index = sectors_per_cluster as usize * cluster_counter * SECTOR_SIZE as usize;
            let end_index = core::cmp::min(output.len(), start_index + (sectors_per_cluster as usize * SECTOR_SIZE as usize * clusters_sequence));
            self.disk.read(start_sector, &mut output[start_index..end_index]);

            next_read_cluster = fat_entry;
            current_cluster = fat_entry;
            cluster_counter += clusters_sequence;
            clusters_sequence = 1;
        }
    }

    /// Write a file to the root dir
    pub fn write_file(&mut self, filename:&str, content:&[u8]){
        log::debug!("Writing file: {}, size: {}", filename, content.len());
        let sectors_per_cluster = self.boot_sector.fat32_bpb.sectors_per_cluster as u32;
        let cluster_size = sectors_per_cluster * SECTOR_SIZE as u32;
        let (name, extension) = self.create_filename(filename).unwrap_or_else(|_|core::panic!("File name format is bad: {}", filename));

        // check if file exists, if exists try to overwrite it, if cant mark it as deleted
        if let ControlFlow::Break(()) = self.handle_existing_filename(name, extension, content) {return}

        // create a new file by allocating place in the root dir and then picking some free fat segment to use it's clusters
        let new_dir_entry = match self.root_dir_cache.as_mut_slice().into_iter().find(|d|d.file_name[0] == DELETED_DIR_ENTRY_PREFIX){
            Some(dir) => {
                dir.file_name = name;
                dir.file_extension = extension;
                dir
            }
            None => {
                // Check the root dir allocation size to check it needs to be reallocated
                let root_dir_allocation_size = (self.root_dir_allocated_clusters_count * self.boot_sector.fat32_bpb.sectors_per_cluster as u32) as usize * SECTOR_SIZE;
                let expected_root_dir_size_after_allocation = (self.root_dir_cache.len() + 1) * size_of::<FatShortDirEntry>();
                if root_dir_allocation_size <= expected_root_dir_size_after_allocation {
                    core::panic!("root dir is too small: {:#X} and driver do not support resizing of the root dir", root_dir_allocation_size);
                }
                // Allocate new entry in the root dir
                self.root_dir_cache.insert(self.root_dir_cache.len() - 1, FatShortDirEntry::new(name, extension, content.len() as u32));    // write to the last place pushing the last item to index len(), in order to keep the DIR_EOF last
                let root_dir_cache_updated_len = self.root_dir_cache.len();

                // Root dir cache len must be at least 2 (entry for the root dir itself and a EOF) and not the one I inserted (so actually 3)
                // This returns the last non EOF entry
                &mut self.root_dir_cache[root_dir_cache_updated_len - 2]
            }
        };
        let required_clusters_count = (content.len() as u32 / cluster_size) + (content.len() as u32 % cluster_size != 0) as u32;
        let free_fat_segment = self.fat_table_cache.as_slice().into_iter().find(|t|t.state == FatSegmentState::Free && t.len >= required_clusters_count).unwrap();
        let first_cluster_index = free_fat_segment.start_index;

        // Update the fat and the root directory
        log::debug!("File first cluster index: {}", first_cluster_index);
        new_dir_entry.set_first_cluster_index(first_cluster_index);
        new_dir_entry.size = content.len() as u32;

        let mut fat_buffer:FatBuffer<SECTOR_SIZE> = FatBuffer::new(self.fat_info, first_cluster_index as usize, Some(required_clusters_count as usize), &mut self.disk);
        for i in 0..required_clusters_count - 1{
            // Adding 1 in order to point to the next fat entry, since all the cluster are allocated in contiguous disk clusters 
            fat_buffer.write(first_cluster_index + i + 1).unwrap();
        }
        fat_buffer.write(FAT_ENTRY_EOF_INDEX).unwrap();

        // write the data to the clusters, since the cluster index is the initial index in the fat I can know which one is free or allocated
        self.write_to_data_section(content, first_cluster_index);

        // sync the modifications
        fat_buffer.flush(&mut self.disk);
        self.write_root_dir_cache();
    }

    fn handle_existing_filename(&mut self, name: [u8; 8], extension: [u8; 3], content: &[u8]) -> ControlFlow<()> {
        if let Some(existing_entry) = self.root_dir_cache.as_mut_slice().into_iter().find(|d|d.file_name == name && d.file_extension == extension){
            log::debug!("File already exists, overwriting it");
            if existing_entry.size == 0 {
                existing_entry.file_name[0] = DELETED_DIR_ENTRY_PREFIX;
                return ControlFlow::Continue(())
            };

            let segment = self.fat_table_cache.as_slice().into_iter().find(|f|f.start_index == existing_entry.get_first_cluster_index()).unwrap().clone();
            let segment_len = match segment.state {
                FatSegmentState::Allocated => segment.len + 1,
                FatSegmentState::AllocatedEof => 1,
                _ => core::panic!("FAT32 FS Error: received not allocated segment"),
            };
            let mut fat_buffer:FatBuffer = FatBuffer::new(self.fat_info, existing_entry.get_first_cluster_index() as usize, Some(segment_len as usize), &mut self.disk);

            // Possible Optimization: Allow more cases to reuse the allocation
            // 1. if its in the range of the cluster alignment
            // 2. If its smaller than the required size (can use some of the allocation)
            if (existing_entry.size as usize) == content.len(){
                log::debug!("Using existing allocation");
                let existing_entry = existing_entry.clone();        // Shadow the original in order to satisfy the borrow checker
                let mut current_cluster = existing_entry.get_first_cluster_index();
                let mut cluster_count = 0;
                while cluster_count < segment_len{
                    self.write_to_data_section(content, current_cluster);
                    current_cluster = fat_buffer.read().ok().unwrap();
                    cluster_count += 1;
                }
                return ControlFlow::Break(());
            }
            else{
                existing_entry.file_name[0] = DELETED_DIR_ENTRY_PREFIX;
                // Mark the fat entries as free
                for _ in 0..segment_len{
                    fat_buffer.write(FAT_ENTRY_FREE_INDEX).ok().unwrap();
                }
                // while fat_buffer.write(FAT_ENTRY_FREE_INDEX).is_ok() {}
                fat_buffer.flush(&mut self.disk);
                // The root dir cache is flushed at the end of the function
            }
        }
        return ControlFlow::Continue(());
    }

    fn write_to_data_section(&mut self, content: &[u8], first_cluster_index: u32) {
        let start_sector = self.get_cluster_start_sector_index(first_cluster_index);
        self.disk.write(start_sector, content);
    }

    fn create_filename(&self, filename:&str)->Result<([u8;8],[u8;3]), ()>{
        const FAT32_ILLEGAL_CHARS:[u8;16] = [b'"', b'*', b'+', b',', b'.', b'/', b':', b';', b'<', b'=', b'>', b'?', b'[',b'\\', b']', b'|' ];
        if filename.len() != FatShortDirEntry::FULL_FILENAME_SIZE || 
            !filename.is_ascii() || 
            filename.as_bytes().into_iter().any(|b|FAT32_ILLEGAL_CHARS.contains(b) && *b > 0x20) || 
            filename.as_bytes().into_iter().any(|c| *c >= b'a' && *c <= b'z'){
            return Err(());
        }
        let raw_filename:[u8;FatShortDirEntry::FULL_FILENAME_SIZE] = filename.as_bytes().try_into().unwrap();
        let name:[u8;FatShortDirEntry::FILE_NAME_SIZE] = raw_filename[.. FatShortDirEntry::FILE_NAME_SIZE].try_into().unwrap();
        let extension:[u8;FatShortDirEntry::FILE_EXTENSION_SIZE] = raw_filename[FatShortDirEntry::FILE_NAME_SIZE ..].try_into().unwrap();
        return Ok((name, extension));
    }

    fn write_root_dir_cache(&mut self){
        let root_sector_index = self.get_cluster_start_sector_index(self.boot_sector.fat32_bpb.root_dir_first_cluster);
        // SAFETY: FatShortDirEntry layout is repr(C)
        let buffer = unsafe{Self::arrayvec_as_buffer(&self.root_dir_cache)};
        self.disk.write(root_sector_index, buffer);
    }

    fn get_cluster_start_sector_index(&self, cluster:u32)->u32{
        const FIRST_DATA_CLUSTER:u32 = 2;

        self.partition_start_sector_index + self.boot_sector.fat32_bpb.reserved_sectors_count as u32 + 
        ((cluster - FIRST_DATA_CLUSTER) * self.boot_sector.fat32_bpb.sectors_per_cluster as u32) + 
        (self.boot_sector.fat32_bpb.sectors_per_fat_32 * self.boot_sector.fat32_bpb.fats_count as u32)
    }
    
    /// Takes an `ArrayVec<T>` and converts it to a byte slice
    /// This is a function in order to borrow the input properly and bind in to the output slice
    /// The function borrows the vec and returns a slice binded to the vec borrow
    /// 
    /// ## SAFETY
    /// T layout must be known
    unsafe fn arrayvec_as_buffer<'a, T, const CAP:usize>(vec:&'a ArrayVec<T, CAP>)->&'a [u8]{
        core::slice::from_raw_parts(vec.as_ptr() as *const u8, vec.len() * core::mem::size_of::<T>())
    }
}