use crate::{
    error::Result,
    fs::file::File,
    scheduler::{SCHEDULER, task::Task},
};
use elf::{ElfBytes, endian::LittleEndian};
use x86_64::PrivilegeLevel;

pub fn spawn(file: &dyn File, privilege: PrivilegeLevel) -> Result<usize> {
    let mut buffer = [0u8; 16_384];
    file.read(&mut buffer)?;

    let elf_data = ElfBytes::<LittleEndian>::minimal_parse(&buffer)?;

    let task = Task::from_elf(elf_data, privilege)?;

    SCHEDULER.lock().spawn(task)
}
