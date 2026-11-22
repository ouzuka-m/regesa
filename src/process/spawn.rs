use crate::{
    error::Result,
    fs::file::File,
    scheduler::{SCHEDULER, task::Task},
};
use elf::{ElfBytes, endian::LittleEndian};

pub fn spawn(file: &dyn File) -> Result<usize> {
    let mut buffer = [0u8; 16_384];
    file.read(&mut buffer)?;

    let elf_data = ElfBytes::<LittleEndian>::minimal_parse(&buffer)?;

    let task = Task::from_elf(elf_data)?;

    SCHEDULER.lock().spawn(task)
}
