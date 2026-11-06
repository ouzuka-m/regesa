use crate::{error::Result, fs::file::File};

pub trait FileSystem<F: File> {
    fn open(&self, path: &str) -> Result<&F>;
}
