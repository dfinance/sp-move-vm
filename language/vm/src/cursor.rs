use core::cmp;

use anyhow::{Error, Result};

pub struct Cursor<T>
where
    T: AsRef<[u8]>,
{
    inner: T,
    pos: u64,
}

impl<T> Cursor<T>
where
    T: AsRef<[u8]>,
{
    pub fn new(inner: T) -> Cursor<T> {
        Cursor { pos: 0, inner }
    }

    pub fn position(&self) -> u64 {
        self.pos
    }

    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let source = self.fill_buf();
        let amt = cmp::min(buf.len(), source.len());

        if amt == 1 {
            buf[0] = source[0];
        } else {
            buf[..amt].copy_from_slice(&source[..amt]);
        }

        self.pos += amt as u64;
        Ok(amt)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let n = buf.len();
        let source = self.fill_buf();

        if buf.len() > source.len() {
            return Err(Error::msg("Unexpected Eof failed to fill whole buffer"));
        }

        if buf.len() == 1 {
            buf[0] = source[0];
        } else {
            buf.copy_from_slice(&source[..n]);
        }

        self.pos += n as u64;
        Ok(())
    }

    fn fill_buf(&mut self) -> &[u8] {
        let amt = cmp::min(self.pos, self.inner.as_ref().len() as u64);
        &self.inner.as_ref()[(amt as usize)..]
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }
}
