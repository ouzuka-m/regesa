use elf::{
    ElfBytes,
    abi::{EM_X86_64, ET_EXEC, PT_LOAD},
    endian::LittleEndian,
    file::Class::ELF64,
};
use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageSize, PageTable, PageTableFlags, Size4KiB,
    },
};

use crate::{
    debug, error,
    error::{Error, Result},
    info,
    memory::{address::PhysExt, allocator::phys::PHYS_ALLOCATOR, mapper, stack::Stack},
    scheduler::{context::Context, lifecycle::exit, rflags::RFLAGS, state::State},
    warn,
};

#[derive(Debug)]
pub struct Task {
    pub context: Context,
    pub state: State,
}

impl Task {
    pub fn new_kernel(entry: fn() -> !) -> Result<Self> {
        let stack = Stack::alloc()?;
        let mut rsp = stack.top();

        rsp -= 8;
        unsafe {
            *(rsp.as_mut_ptr()) = entry as usize;
        }

        rsp -= 8;
        unsafe {
            *(rsp.as_mut_ptr()) = RFLAGS as usize;
        }

        let (current_frame, _) = Cr3::read();
        let cr3 = current_frame.start_address();

        let context = Context::new(rsp.as_u64(), cr3.as_u64());
        let state = State::Ready;

        Ok(Self { context, state })
    }

    pub fn from_elf(elf_data: ElfBytes<LittleEndian>) -> Result<Self> {
        info!(
            "Loading ELF binary (entry point: {:#x})",
            elf_data.ehdr.e_entry
        );

        let ehdr = elf_data.ehdr;
        if ehdr.e_type != ET_EXEC || ehdr.class != ELF64 || ehdr.e_machine != EM_X86_64 {
            error!(
                "Invalid ELF header: e_type={:?}, class={:?}, e_machine={:?}",
                ehdr.e_type, ehdr.class, ehdr.e_machine
            );
            return Err(Error::NotExecutable);
        }

        if let Some(phdrs) = elf_data.segments() {
            let mut frame_allocator = PHYS_ALLOCATOR.lock();

            let new_frame = frame_allocator.allocate_frame().ok_or_else(|| {
                error!("Failed to allocate PML4 frame for ELF task");
                Error::OutOfMemory
            })?;

            let new_frame_addr = new_frame.start_address();
            debug!("Allocated new PML4 frame at {:#x}", new_frame_addr.as_u64());
            let new_pml4 = unsafe { &mut *(new_frame_addr.to_virt().as_mut_ptr::<PageTable>()) };

            new_pml4.zero();

            let (current_frame, _) = Cr3::read();
            let current_pml4 = unsafe {
                &*current_frame
                    .start_address()
                    .to_virt()
                    .as_ptr::<PageTable>()
            };

            for i in 256..512 {
                new_pml4[i] = current_pml4[i].clone();
            }

            let mut mapper = mapper::from_pml4(new_pml4);

            for phdr in phdrs {
                if phdr.p_type != PT_LOAD {
                    continue;
                }

                let p_vaddr = phdr.p_vaddr;
                let p_flags = phdr.p_flags;

                let p_memsz = phdr.p_memsz;
                let p_filesz = phdr.p_filesz;

                debug!(
                    "Loading PT_LOAD segment: vaddr={:#x}, filesz={:#x}, memsz={:#x}, flags={:#x}",
                    p_vaddr, p_filesz, p_memsz, p_flags
                );

                let mut flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

                if p_flags & 1 == 0 {
                    flags |= PageTableFlags::NO_EXECUTE;

                    warn!("Loading RWX segment! This is insecure, W^X violated");
                }

                let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(p_vaddr));
                let end_vaddr = VirtAddr::new(p_vaddr + p_memsz);

                let end_page = Page::<Size4KiB>::containing_address(if p_memsz > 0 {
                    end_vaddr - 1
                } else {
                    end_vaddr
                });
                let page_count = end_page - start_page + 1;

                debug!(
                    "Mapping {} pages from {:?} to {:?}",
                    page_count,
                    start_page.start_address(),
                    end_page.start_address()
                );

                let data_ptr = elf_data
                    .segment_data(&phdr)
                    .map_err(|e| {
                        error!("Failed to get segment data: {:?}", e);
                        Error::InvalidSegment
                    })?
                    .as_ptr();

                for i in 0..page_count {
                    let page = start_page + i;
                    let frame = frame_allocator.allocate_frame().ok_or_else(|| {
                        error!(
                            "Failed to allocate physical frame for page {:?}",
                            page.start_address()
                        );
                        Error::OutOfMemory
                    })?;

                    unsafe {
                        mapper
                            .map_to(page, frame, flags, &mut *frame_allocator)
                            .map_err(|e| {
                                error!(
                                    "Failed to map page {:?} to frame {:?}: {:?}",
                                    page.start_address(),
                                    frame.start_address(),
                                    e
                                );
                                Error::MemoryMapFailed
                            })?
                            .ignore();
                    }

                    let frame_virt = frame.start_address().to_virt().as_mut_ptr::<u8>();

                    unsafe {
                        core::ptr::write_bytes(frame_virt, 0, Size4KiB::SIZE as usize);

                        let page_offset = (i * Size4KiB::SIZE) as usize;
                        if page_offset < p_filesz as usize {
                            let bytes_to_copy = core::cmp::min(
                                Size4KiB::SIZE as usize,
                                p_filesz as usize - page_offset,
                            );

                            core::ptr::copy_nonoverlapping(
                                data_ptr.add(page_offset),
                                frame_virt,
                                bytes_to_copy,
                            );
                        }
                    }
                }
            }

            drop(frame_allocator);

            let entry: extern "C" fn() = unsafe { core::mem::transmute(ehdr.e_entry) };

            let stack = Stack::alloc().inspect_err(|e| {
                error!("Failed to allocate stack for ELF task: {:?}", e);
            })?;
            let mut rsp = stack.top();

            rsp -= 8;
            unsafe {
                *(rsp.as_mut_ptr()) = (exit::exit as fn() -> !) as usize;
            }

            rsp -= 8;
            unsafe {
                *(rsp.as_mut_ptr()) = entry as usize;
            }

            rsp -= 8;
            unsafe {
                *(rsp.as_mut_ptr()) = RFLAGS as usize;
            }

            let context = Context::new(rsp.as_u64(), new_frame_addr.as_u64());
            let state = State::Ready;

            info!(
                "Successfully loaded ELF task (entry={:#x}, rsp={:#x}, cr3={:#x})",
                ehdr.e_entry,
                rsp.as_u64(),
                new_frame_addr.as_u64()
            );

            Ok(Self { context, state })
        } else {
            Err(Error::MissingProgramHeaders)
        }
    }
}
