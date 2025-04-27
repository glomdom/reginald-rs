#![no_main]
#![no_std]

mod colors;
mod elf;
mod header;
mod paging;
mod utilities;

use core::arch::asm;

use colors::{clear, set_fg_color};
use elf::{copy_load_headers, slice_to_elf_header};
use header::print_header;

use log::{info, trace};
use paging::{load_cr3, setup_paging};
use uefi::{
    boot::MemoryType,
    prelude::*,
    proto::{
        console::{gop::GraphicsOutput, text::Color},
        loaded_image::LoadedImage,
        media::{
            file::{File, FileAttribute, FileMode},
            fs::SimpleFileSystem,
        },
    },
};

use utilities::{get_protocol_from_handle, get_shared_protocol};
use utilities::{open_root_dir, read_from_regular_file};

#[repr(C)]
#[derive(Debug)]
pub struct FramebufferInfo {
    pub buffer: *mut u8,
    pub size: usize,
    pub stride: usize,
    pub width: usize,
    pub height: usize,
    pub pixel_format: u32,
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    set_fg_color(Color::White);
    clear();
    print_header();

    let loaded_image = get_protocol_from_handle::<LoadedImage>(boot::image_handle());
    let device_handle = loaded_image
        .device()
        .expect("loaded image does not have a device handle");

    let fs = get_protocol_from_handle::<SimpleFileSystem>(device_handle);
    let mut root = open_root_dir(fs);
    let mut kf = root
        .open(
            cstr16!("\\EFI\\BOOT\\kernel.elf"),
            FileMode::Read,
            FileAttribute::empty(),
        )
        .expect("failed to open kernel file")
        .into_regular_file()
        .expect("kernel.elf is not a regular file");

    let kernel_data = read_from_regular_file(&mut kf);
    let elf_hdr = slice_to_elf_header(&kernel_data);

    let mut gop = get_shared_protocol::<GraphicsOutput>();

    let mode = gop.current_mode_info();
    let mut fb = gop.frame_buffer();

    let fb_info = FramebufferInfo {
        buffer: fb.as_mut_ptr(),
        size: fb.size(),
        stride: mode.stride(),
        width: mode.resolution().0 as usize,
        height: mode.resolution().1 as usize,
        pixel_format: mode.pixel_format() as u32,
    };

    trace!("{:#?}", fb_info);

    let reloc_delta = copy_load_headers(&kernel_data, elf_hdr);

    let paging = unsafe { setup_paging() };
    unsafe { load_cr3(paging.pml4_phys_addr) }

    trace!("enabled paging");

    let _final_mem_map = unsafe { boot::exit_boot_services(MemoryType::LOADER_DATA) };

    let entry_addr = (elf_hdr.e_entry as isize + reloc_delta) as usize;
    let kernel_start: extern "C" fn(*const FramebufferInfo) -> ! =
        unsafe { core::mem::transmute(entry_addr) };

    kernel_start(&fb_info as *const _);
}
