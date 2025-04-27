use core::{ptr, slice};

use log::trace;
use uefi::{
    boot::{self, MemoryType, ScopedProtocol},
    proto::media::{
        file::{Directory, File, FileInfo, RegularFile},
        fs::SimpleFileSystem,
    },
};

pub fn open_root_dir(mut fs_proto: ScopedProtocol<SimpleFileSystem>) -> Directory {
    trace!("OPEN root directory");

    fs_proto
        .open_volume()
        .expect("failed to open root directory")
}

pub fn read_from_regular_file(file: &mut RegularFile) -> &mut [u8] {
    let mut file_info_buffer: [u8; 256] = [0; 256];
    let file_info = file
        .get_info::<FileInfo>(&mut file_info_buffer)
        .expect("failed to get file info");

    trace!("got file info");

    let file_size = file_info.file_size().try_into().expect("file is too large");
    let file_data_buffer_ptr = boot::allocate_pool(MemoryType::LOADER_DATA, file_size)
        .expect("failed to allocate memory for file")
        .as_ptr();

    trace!("allocated data for file at {:?}", file_data_buffer_ptr);

    let file_data_buffer = unsafe {
        ptr::write_bytes(file_data_buffer_ptr, 0, file_size);
        slice::from_raw_parts_mut(file_data_buffer_ptr, file_size)
    };

    file.read(file_data_buffer).expect("failed to write data to buffer");

    trace!("wrote data successfully");

    file_data_buffer
}
