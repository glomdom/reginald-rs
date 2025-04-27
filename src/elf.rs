use core::{ptr, slice};
use log::trace;
use uefi::boot::{self, AllocateType, MemoryType};

#[repr(C)]
#[derive(Debug)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

/// Interpret the start of the given byte slice as an ELF header
pub fn slice_to_elf_header(data: &[u8]) -> &Elf64Header {
    use core::mem::size_of;
    assert!(data.len() >= size_of::<Elf64Header>());
    if data[0..4] != [0x7f, 0x45, 0x4c, 0x46] {
        panic!("invalid elf file");
    }
    unsafe { &*(data.as_ptr() as *const Elf64Header) }
}

/// Load each PT_LOAD segment, trying Address first, falling back to Any and recording delta
pub fn copy_load_headers(elf_data: &[u8], elf_header: &Elf64Header) -> isize {
    let phs = unsafe {
        slice::from_raw_parts(
            (elf_data.as_ptr() as usize + elf_header.e_phoff as usize) as *const Elf64ProgramHeader,
            elf_header.e_phnum as usize,
        )
    };

    // Cumulative relocation offset (actual_base - expected_base)
    let mut reloc_offset: isize = 0;

    for ph in phs {
        if ph.p_type != 1 {
            continue;
        }

        let src_off = ph.p_offset as usize;
        let file_sz = ph.p_filesz as usize;
        let mem_sz  = ph.p_memsz  as usize;
        let pages   = (mem_sz + 0x1000 - 1) / 0x1000;

        // Try fixed-address allocation
        let actual_ptr = match boot::allocate_pages(
            AllocateType::Address(ph.p_vaddr),
            MemoryType::LOADER_DATA,
            pages as usize,
        ) {
            Ok(_) => {
                // Mapped at the intended virtual address
                trace!("allocated {} pages at {:#x}", pages, ph.p_vaddr);
                ph.p_vaddr as *mut u8
            }
            Err(_) => {
                // Fallback: allocate anywhere
                let base = boot::allocate_pages(
                    AllocateType::AnyPages,
                    MemoryType::LOADER_DATA,
                    pages as usize,
                )
                .expect("AllocateType::Any failed");

                // Compute delta
                reloc_offset = base.as_ptr() as isize - ph.p_vaddr as isize;
                trace!(
                    "fallback alloc: {} pages at {:?}, delta {:+#x}",
                    pages,
                    base.as_ptr(),
                    reloc_offset
                );
                base.as_ptr() as *mut u8
            }
        };

        // Copy and zero extend
        unsafe {
            ptr::copy_nonoverlapping(elf_data.as_ptr().add(src_off), actual_ptr, file_sz);
            if mem_sz > file_sz {
                ptr::write_bytes(actual_ptr.add(file_sz), 0, mem_sz - file_sz);
            }
        }

        trace!("copied {} bytes to {:p}", file_sz, actual_ptr);
    }

    reloc_offset
}
