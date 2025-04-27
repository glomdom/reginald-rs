use uefi::boot::{self, AllocateType, MemoryType};
use core::{arch::asm};
use log::trace;

/// Constants
const PAGE_SIZE: usize = 4096;
const ENTRY_COUNT: usize = 512;
/// Base of the higher-half virtual region (canonical sign-extended)
const HIGHER_HALF_BASE: u64 = 0xffff_8000_0000_0000;
/// Compute PML4 index for mapping the higher-half region (bits 47:39)
const KERNEL_PML4_INDEX: usize = ((HIGHER_HALF_BASE >> 39) & 0x1ff) as usize;

/// Page Table Entry flags
const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const HUGE_PAGE: u64 = 1 << 7;

#[repr(align(4096))]
#[repr(C)]
struct PageTable([u64; ENTRY_COUNT]);

/// Paging context, returns the physical PML4 address
pub struct PagingContext {
    pub pml4_phys_addr: u64,
}

/// Allocate a zeroed 4KiB page; UEFI picks the address for us
unsafe fn alloc_page() -> *mut PageTable {
    let pages = boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).expect("failed to allocate page table");
    let ptr = pages.as_ptr() as *mut PageTable;
    // Zero exactly one 4KiB page
    ptr.write_bytes(0, 1);
    ptr
}

/// Build a 4-level page table that:
/// 1) Identity-maps 0..2GiB using 2MiB huge pages
/// 2) Mirrors that same 0..2GiB at the higher-half base
/// Then loads CR3 with the new PML4.
pub unsafe fn setup_paging() -> PagingContext {
    trace!("Setting up 4-level paging with higher-half map");

    // Allocate top-level tables
    let pml4     = alloc_page();
    let pdpt_id  = alloc_page();
    let pdpt_hh  = alloc_page();
    // Two PDs for 0..2GiB identity, two for higher-half
    let pd0      = alloc_page();
    let pd1      = alloc_page();
    let pd_hh0   = alloc_page();
    let pd_hh1   = alloc_page();

    // Link PML4 entries
    (*pml4).0[0]                  = (pdpt_id as u64) | PRESENT | WRITABLE;
    (*pml4).0[KERNEL_PML4_INDEX] = (pdpt_hh as u64) | PRESENT | WRITABLE;

    // Link PDPT for identity
    (*pdpt_id).0[0] = (pd0 as   u64) | PRESENT | WRITABLE;
    (*pdpt_id).0[1] = (pd1 as   u64) | PRESENT | WRITABLE;
    // Link PDPT for higher-half
    (*pdpt_hh).0[0] = (pd_hh0 as u64) | PRESENT | WRITABLE;
    (*pdpt_hh).0[1] = (pd_hh1 as u64) | PRESENT | WRITABLE;

    // Populate all four PDs with 2MiB huge pages
    for i in 0..ENTRY_COUNT {
        let base_low  = (i as u64) * 2 * 1024 * 1024;
        let base_high = base_low + (ENTRY_COUNT as u64) * 2 * 1024 * 1024; // +1GiB

        (*pd0).   0[i] = base_low  | PRESENT | WRITABLE | HUGE_PAGE;
        (*pd_hh0).0[i] = base_low  | PRESENT | WRITABLE | HUGE_PAGE;
        (*pd1).   0[i] = base_high | PRESENT | WRITABLE | HUGE_PAGE;
        (*pd_hh1).0[i] = base_high | PRESENT | WRITABLE | HUGE_PAGE;
    }

    // Activate the new tables
    trace!("Reloading CR3 with PML4 at {:#x}", pml4 as u64);
    load_cr3(pml4 as u64);

    PagingContext { pml4_phys_addr: pml4 as u64 }
}

/// Writes to CR3 to switch page tables
pub unsafe fn load_cr3(pml4_phys: u64) {
    asm!("mov cr3, {}", in(reg) pml4_phys, options(nostack, preserves_flags));
}
