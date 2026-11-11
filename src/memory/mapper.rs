//! Virtual memory page-table mapper.
//!
//! Reads the active Level-4 page table from `CR3` and wraps it in an
//! [`OffsetPageTable`] using the HHDM offset so the kernel can map and
//! unmap virtual pages.

use x86_64::{
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
};

use crate::{boot::info::HHDM, memory::address::PhysExt};

pub fn current() -> OffsetPageTable<'static> {
    let (level_4_frame, _) = Cr3::read();

    let phys = level_4_frame.start_address();
    let virt = phys.to_virt();

    // SAFETY: The virtual address is derived from the CR3 physical address
    // via the HHDM offset, which the bootloader guarantees to be a valid
    // identity mapping of all physical memory. The mutable reference is
    // safe because we are the only consumer during early boot.
    unsafe {
        let level_4_table = &mut *virt.as_mut_ptr();

        OffsetPageTable::new(level_4_table, *HHDM)
    }
}

pub fn from_pml4(level_4_table: &mut PageTable) -> OffsetPageTable<'_> {
    unsafe { OffsetPageTable::new(level_4_table, *HHDM) }
}
