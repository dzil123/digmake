use std::io::BufRead;
use std::io::Read;

pub struct CountingRead<'a, T: BufRead> {
    pub amt_read: usize,
    pub inner: &'a mut T,
}

impl<'a, T: BufRead> CountingRead<'a, T> {
    fn new(inner: &'a mut T) -> Self {
        CountingRead { amt_read: 0, inner }
    }
}

impl<'a, T: BufRead> Read for CountingRead<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        self.inner.read(buf)
    }
}

impl<'a, T: BufRead> BufRead for CountingRead<'a, T> {
    fn fill_buf(&mut self) -> std::result::Result<&[u8], std::io::Error> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.amt_read += amt;
        self.inner.consume(amt);
    }
}

pub fn count_reads<T, U, F>(inner: &mut T, func: F) -> (U, usize)
where
    T: BufRead,
    F: FnOnce(&mut CountingRead<T>) -> U,
{
    let mut this = CountingRead::new(inner);
    let result = func(&mut this);
    (result, this.amt_read)
}
