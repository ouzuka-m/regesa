use crate::{
    error::{Error, Result},
    fs::file::File,
    memory::{frame_allocator::FRAME_ALLOCATOR, mapper},
    scheduler::SCHEDULER,
};
use elf::{
    ElfBytes,
    abi::{EM_X86_64, ET_EXEC, PT_LOAD},
    endian::LittleEndian,
    file::Class,
};
use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
};

pub fn spawn(file: &dyn File) -> Result<usize> {
    let mut buffer = [0u8; 8192];
    file.read(&mut buffer)?;
    let elf_bytes =
        ElfBytes::<LittleEndian>::minimal_parse(&buffer).map_err(|_| Error::NotExecutable)?;

    let ehdr = elf_bytes.ehdr;
    if ehdr.e_type != ET_EXEC || ehdr.class != Class::ELF64 || ehdr.e_machine != EM_X86_64 {
        return Err(Error::NotExecutable);
    }

    if let Some(phdrs) = elf_bytes.segments() {
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        for phdr in phdrs {
            if phdr.p_type != PT_LOAD {
                continue;
            }

            let p_vaddr = phdr.p_vaddr;
            let p_flags = phdr.p_flags;

            let p_memsz = phdr.p_memsz;
            let p_filesz = phdr.p_filesz;

            let mut flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            if p_flags & 1 == 0 {
                flags |= PageTableFlags::NO_EXECUTE;
            }

            let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(p_vaddr));
            let end_vaddr = VirtAddr::new(p_vaddr + p_memsz);

            let end_page = Page::<Size4KiB>::containing_address(if p_memsz > 0 {
                end_vaddr - 1
            } else {
                end_vaddr
            });
            let page_count = end_page - start_page + 1;

            for i in 0..page_count {
                let page = start_page + i;
                let frame = frame_allocator.allocate_frame().ok_or(Error::OutOfMemory)?;

                unsafe {
                    mapper::current()
                        .map_to(page, frame, flags, &mut *frame_allocator)
                        .map_err(|_| Error::MemoryMapFailed)?
                        .flush();
                }
            }

            let src = elf_bytes
                .segment_data(&phdr)
                .map_err(|_| Error::InvalidSegment)?
                .as_ptr();
            let dst = p_vaddr as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src, dst, p_filesz as usize);
            }

            // BSS Zeroing
            if p_memsz > p_filesz {
                let start = unsafe { dst.add(p_filesz as usize) };
                let end = (p_memsz - p_filesz) as usize;

                unsafe { core::ptr::write_bytes(start, 0x00, end) };
            }
        }

        drop(frame_allocator);

        let entry_fn: fn() = unsafe { core::mem::transmute(ehdr.e_entry) };

        SCHEDULER.lock().spawn(entry_fn)
    } else {
        Err(Error::MissingProgramHeaders)
    }
}
