use alloc::vec::Vec;

use crate::{
    error::{Error, Result},
    fs::{file::File, filesystem::FileSystem},
};

pub const BLOCK: usize = 512;

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
}

impl<'a> FileSystem<Entry<'a>> for Tar<'a> {
    fn open(&self, path: &str) -> Result<&Entry<'a>> {
        self.entries
            .iter()
            .find(|entry| entry.name() == path)
            .ok_or(Error::NotFound)
    }
}

#[derive(Debug)]
pub struct Entry<'a> {
    name: &'a str,
    data: &'a [u8],
    size: u64,
}

impl<'a> Entry<'a> {
    pub fn new(name: &'a str, data: &'a [u8], size: u64) -> Self {
        Entry { name, data, size }
    }
}

impl<'a> File for Entry<'a> {
    fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        let len = self.data.len().min(buffer.len());

        buffer[..len].copy_from_slice(&self.data[..len]);

        Ok(len)
    }

    fn name(&self) -> &str {
        self.name
    }

    fn size(&self) -> u64 {
        self.size
    }
}
