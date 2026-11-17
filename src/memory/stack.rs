use x86_64::{
    VirtAddr,
    structures::paging::{PageSize, Size4KiB},
};

use crate::{
    error::Result,
    memory::{address::PhysExt, allocator},
};

#[derive(Debug)]
pub struct Stack {
    top: VirtAddr,
}

impl Stack {
    pub fn alloc() -> Result<Self> {
        let frame = allocator::phys::allocate()?;

        let top = frame.start_address().to_virt() + Size4KiB::SIZE;

        Ok(Self { top })
    }

    pub const fn top(&self) -> VirtAddr {
        self.top
    }
}
