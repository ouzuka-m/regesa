use alloc::vec::Vec;
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

use crate::{
    initramfs::{entry::Entry, error::FsError},
    memory::{frame_allocator::FRAME_ALLOCATOR, mapper},
};

#[derive(Debug)]
pub struct Tar<'a> {
    entries: Vec<Entry<'a>>,
}

impl<'a> Tar<'a> {
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(5),
        }
    }

    pub fn push(&mut self, entry: Entry<'a>) {
        self.entries.push(entry);
    }

    pub fn read(&'a self, path: &'a str) -> Result<&'a [u8], FsError> {
        self.entries
            .iter()
            .find(|entry| entry.name() == path)
            .map(|entry| entry.data())
            .ok_or(FsError::NotFound)
    }

    pub fn spawn(&self, path: &'a str) -> Result<usize, FsError> {
        let data = self.read(path)?;
        let elf_bytes =
            ElfBytes::<LittleEndian>::minimal_parse(data).map_err(|_| FsError::NotExecutable)?;

        let header = elf_bytes.ehdr;
        if header.e_type != ET_EXEC || header.class != Class::ELF64 || header.e_machine != EM_X86_64
        {
            return Err(FsError::NotExecutable);
        }

        if let Some(phdrs) = elf_bytes.segments() {
            let mut frame_allocator = FRAME_ALLOCATOR.lock();
            for phdr in phdrs {
                if phdr.p_type != PT_LOAD {
                    continue;
                }

                let mut flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

                if phdr.p_flags & 1 == 0 {
                    flags |= PageTableFlags::NO_EXECUTE;
                }

                let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(phdr.p_vaddr));
                let end_vaddr = VirtAddr::new(phdr.p_vaddr + phdr.p_memsz);

                let end_page = Page::<Size4KiB>::containing_address(if phdr.p_memsz > 0 {
                    end_vaddr - 1u64
                } else {
                    end_vaddr
                });
                let page_count = end_page - start_page + 1;

                for i in 0..page_count {
                    let page = start_page + i;
                    let frame = frame_allocator
                        .allocate_frame()
                        .ok_or(FsError::NoUsableMemory)?;

                    unsafe {
                        mapper::current()
                            .map_to(page, frame, flags, &mut *frame_allocator)
                            .map_err(|_| FsError::MapperError)?
                            .flush();
                    }
                }

                let raw_data = elf_bytes
                    .segment_data(&phdr)
                    .map_err(|_| FsError::InvalidSegmentData)?;
                let vaddr = phdr.p_vaddr as *mut u8;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        raw_data.as_ptr(),
                        vaddr,
                        phdr.p_filesz as usize,
                    );
                }

                if phdr.p_memsz > phdr.p_filesz {
                    let start = unsafe { vaddr.add(phdr.p_filesz as usize) };
                    let end = (phdr.p_memsz - phdr.p_filesz) as usize;

                    unsafe { core::ptr::write_bytes(start, 0x00, end) };
                }
            }

            let entry_point = header.e_entry;

            let entry_fn: extern "C" fn() -> u64 = unsafe { core::mem::transmute(entry_point) };

            let rax_result = entry_fn();

            crate::info!("Return: {}", rax_result);
        }

        Ok(0)
    }
}
