use x86_64::{
    VirtAddr,
    instructions::interrupts,
    structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB},
};

use crate::{
    error::{Error, Result},
    memory::{address::PhysExt, frame_allocator::FRAME_ALLOCATOR},
};

#[derive(Debug)]
pub struct Stack {
    frame: PhysFrame,
    top: VirtAddr,
}

impl Stack {
    pub fn alloc() -> Result<Self> {
        let frame = interrupts::without_interrupts(|| FRAME_ALLOCATOR.lock().allocate_frame())
            .ok_or(Error::OutOfMemory)?;

        let top = frame.start_address().to_virt() + Size4KiB::SIZE;

        Ok(Self { frame, top })
    }

    pub fn deallocate(&self) {
        unsafe { FRAME_ALLOCATOR.lock().deallocate_frame(self.frame()) };
    }

    pub const fn frame(&self) -> PhysFrame {
        self.frame
    }

    pub const fn top(&self) -> VirtAddr {
        self.top
    }
}
