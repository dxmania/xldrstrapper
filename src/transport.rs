use std::time::Duration;

use crate::Result;

pub trait Transport {
    fn read(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize>;
    fn write(&mut self, data: &[u8], timeout: Duration) -> Result<()>;
}