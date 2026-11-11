use crate::{
    error::{Error, Result},
    fs::file::File,
    scheduler::{SCHEDULER, task::Task},
};
use elf::{ElfBytes, endian::LittleEndian};

pub fn spawn(file: &dyn File) -> Result<usize> {
    let mut buffer = [0u8; 8192];
    file.read(&mut buffer)?;

    let elf_data =
        ElfBytes::<LittleEndian>::minimal_parse(&buffer).map_err(|_| Error::NotExecutable)?;

    let task = Task::from_elf(elf_data)?;

    SCHEDULER.lock().spawn(task)
}
