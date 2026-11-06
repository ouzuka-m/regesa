use crate::error::Result;

pub trait File {
    fn read(&self, buffer: &mut [u8]) -> Result<usize>;

    fn name(&self) -> &str;

    fn size(&self) -> u64;
}
