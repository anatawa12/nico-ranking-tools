use std::io::{Read, IoSliceMut};
use indicatif::ProgressBar;
use std::io::Result;

pub struct ProgressReader<'p, R : Read> {
    inner: R,
    progress: &'p ProgressBar,
}

impl <'p, R: Read> ProgressReader<'p, R> {
    pub fn new(progress: &'p ProgressBar, inner: R) -> Self {
        Self {
            inner,
            progress,
        }
    }
}

impl <'p, R: Read> Read for ProgressReader<'p, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let size = self.inner.read(buf)?;
        self.progress.inc(size as u64);
        Ok(size)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> {
        let size = self.inner.read_vectored(bufs)?;
        self.progress.inc(size as u64);
        Ok(size)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        let size = self.inner.read_to_string(buf)?;
        self.progress.inc(size as u64);
        Ok(size)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.inner.read_exact(buf)?;
        self.progress.inc(buf.len() as u64);
        Ok(())
    }
}
