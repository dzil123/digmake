use std::io::Read;

// only works if you call .read() lol, everything else doesnt go through here
pub struct CountingRead<'a, T: Read> {
    pub amt_read: usize,
    pub inner: &'a mut T,
}

impl<'a, T: Read> CountingRead<'a, T> {
    fn new(inner: &'a mut T) -> Self {
        CountingRead { amt_read: 0, inner }
    }
}

impl<'a, T: Read> Read for CountingRead<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        let amt = self.inner.read(buf)?;
        self.amt_read += amt;
        Ok(amt)
    }
}

pub fn count_reads<T, U, F>(inner: &mut T, func: F) -> (U, usize)
where
    T: Read,
    F: FnOnce(&mut CountingRead<T>) -> U,
{
    let mut this = CountingRead::new(inner);
    let result = func(&mut this);
    (result, this.amt_read)
}
