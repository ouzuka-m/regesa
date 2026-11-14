//! Global kernel heap allocator.
//!
//! Maps a contiguous range of virtual pages to physical frames and
//! initialises a buddy-system allocator as the `#[global_allocator]`.

use buddy_system_allocator::LockedHeap;
use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
};

use crate::memory::{allocator::phys::PHYS_ALLOCATOR, mapper};

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap<33> = LockedHeap::empty();

pub const HEAP_START: usize = 0xFFFF_9000_0000_0000;
pub const HEAP_SIZE: usize = 1024 * 1024; // 1 MB, 256 pages

pub fn init() {
    let mut mapper = mapper::current();

    let page_range = {
        let start = VirtAddr::new(HEAP_START as u64);

        // Subtract 1 to prevent use of page 257
        let end = start + HEAP_SIZE as u64 - 1u64;

        let start_page: Page<Size4KiB> = Page::containing_address(start);
        let end_page: Page<Size4KiB> = Page::containing_address(end);

        Page::range_inclusive(start_page, end_page)
    };

    let mut frame_allocator = PHYS_ALLOCATOR.lock();

    for page in page_range {
        let frame = frame_allocator.allocate_frame().expect("No usable memory");
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
        // SAFETY: The page is in the dedicated heap virtual range and has
        // not been previously mapped. The frame is freshly allocated and
        // the flags grant read/write access with no-execute.
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut *frame_allocator)
                .expect("Failed to map page & frame")
                .flush();
        }
    }

    drop(frame_allocator);

    // SAFETY: The virtual address range [HEAP_START .. HEAP_START + HEAP_SIZE)
    // has just been identity-mapped to valid physical frames above, so it is
    // safe to hand this region to the buddy allocator.
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    crate::info!(
        "Heap allocator initialized at {:#X} ({} KB)",
        HEAP_START,
        HEAP_SIZE / 1024
    );
}
