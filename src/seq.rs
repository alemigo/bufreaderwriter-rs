use std::io::{self, BufReader, BufWriter, IntoInnerError, Read, Write};

enum BufIO<RW: Read + Write> {
    Reader(BufReader<RW>),
    Writer(BufWriter<RW>),
}

impl<RW: Read + Write> BufIO<RW> {
    fn new_writer(rw: RW, capacity: Option<usize>) -> BufIO<RW> {
        BufIO::Writer(match capacity {
            Some(c) => BufWriter::with_capacity(c, rw),
            None => BufWriter::new(rw),
        })
    }

    fn new_reader(rw: RW, capacity: Option<usize>) -> BufIO<RW> {
        BufIO::Reader(match capacity {
            Some(c) => BufReader::with_capacity(c, rw),
            None => BufReader::new(rw),
        })
    }

    fn get_mut(&mut self) -> &mut RW {
        match self {
            BufIO::Reader(r) => r.get_mut(),
            BufIO::Writer(w) => w.get_mut(),
        }
    }

    fn get_ref(&self) -> &RW {
        match self {
            BufIO::Reader(r) => r.get_ref(),
            BufIO::Writer(w) => w.get_ref(),
        }
    }

    fn into_inner(self) -> Result<RW, IntoInnerError<BufWriter<RW>>> {
        match self {
            BufIO::Reader(r) => Ok(r.into_inner()),
            BufIO::Writer(w) => Ok(w.into_inner()?),
        }
    }

    fn capacity(&self) -> usize {
        match self {
            BufIO::Reader(r) => r.capacity(),
            BufIO::Writer(w) => w.capacity(),
        }
    }
}

pub struct BufReaderWriterSeq<RW: Read + Write> {
    inner: Option<BufIO<RW>>,
    buffer: Option<Box<Vec<u8>>>,
    pos: usize,
    capacity: Option<usize>,
}

impl<RW: Read + Write> BufReaderWriterSeq<RW> {
    /// Returns a new BufReaderWriterSeq instance, expecting a write as the first operation.
    pub fn new_writer(rw: RW) -> BufReaderWriterSeq<RW> {
        BufReaderWriterSeq {
            inner: Some(BufIO::new_writer(rw, None)),
            buffer: None,
            pos: 0,
            capacity: None,
        }
    }

    /// Returns a new BufReaderWriterSeq instance, expecting a write as the first operation, with specified buffer capacity.
    pub fn writer_with_capacity(capacity: usize, rw: RW) -> BufReaderWriterSeq<RW> {
        BufReaderWriterSeq {
            inner: Some(BufIO::new_writer(rw, Some(capacity))),
            buffer: None,
            pos: 0,
            capacity: Some(capacity),
        }
    }

    /// Returns a new BufReaderWriter instance, expecting a read as the first operation.
    pub fn new_reader(rw: RW) -> BufReaderWriterSeq<RW> {
        BufReaderWriterSeq {
            inner: Some(BufIO::new_reader(rw, None)),
            buffer: None,
            pos: 0,
            capacity: None,
        }
    }

    /// Returns a new BufReaderWriter instance, expecting a read as the first operation, with specified buffer capacity.
    pub fn reader_with_capacity(capacity: usize, rw: RW) -> BufReaderWriterSeq<RW> {
        BufReaderWriterSeq {
            inner: Some(BufIO::new_reader(rw, Some(capacity))),
            buffer: None,
            pos: 0,
            capacity: Some(capacity),
        }
    }

    /// Gets a mutable reference to the underlying reader/writer.
    pub fn get_mut(&mut self) -> &mut RW {
        self.inner.as_mut().unwrap().get_mut()
    }

    /// Gets a reference to the underlying reader/writer.
    pub fn get_ref(&self) -> &RW {
        self.inner.as_ref().unwrap().get_ref()
    }

    /// Unwraps this `BufReaderWriter`, returning the underlying reader/writer.  Note: the `BufReaderWriter` should be dropped after using this.
    pub fn into_inner(self) -> Result<RW, IntoInnerError<BufWriter<RW>>> {
        self.inner.unwrap().into_inner()
    }

    /// Returns true if the `BufReaderWriter` in read mode, otherwise false for write mode.
    pub fn is_reader(&self) -> bool {
        match self.inner.as_ref().unwrap() {
            BufIO::Reader(_) => true,
            _ => false,
        }
    }

    /// Gets a reference to the underlying buffered reader, available if in read mode.
    pub fn get_bufreader_ref(&self) -> Option<&BufReader<RW>> {
        match self.inner.as_ref().unwrap() {
            BufIO::Reader(r) => Some(r),
            _ => None,
        }
    }

    /// Gets a mutable reference to the underlying buffered reader, available if in read mode.
    pub fn get_bufreader_mut(&mut self) -> Option<&mut BufReader<RW>> {
        match self.inner.as_mut().unwrap() {
            BufIO::Reader(r) => Some(r),
            _ => None,
        }
    }

    /// Unwraps this `BufReaderWriter` returning the BufReader, available if in read mode.  Note: the `BufReaderWriter` should be dropped after using this.
    pub fn into_bufreader(self) -> Option<BufReader<RW>> {
        match self.inner.unwrap() {
            BufIO::Reader(r) => Some(r),
            _ => None,
        }
    }

    /// Gets a reference to the underlying buffered writer, available if in write mode.
    pub fn get_bufwriter_ref(&self) -> Option<&BufWriter<RW>> {
        match self.inner.as_ref().unwrap() {
            BufIO::Writer(w) => Some(w),
            _ => None,
        }
    }

    /// Gets a mutable reference to the underlying buffered writer, available if in write mode.
    pub fn get_bufwriter_mut(&mut self) -> Option<&mut BufWriter<RW>> {
        match self.inner.as_mut().unwrap() {
            BufIO::Writer(w) => Some(w),
            _ => None,
        }
    }

    /// Unwraps this `BufReaderWriter` returning the `BufWriter`, available if in read mode.  Note: the `BufReaderWriter` should be dropped after using this.
    pub fn into_bufwriter(self) -> Option<BufWriter<RW>> {
        match self.inner.unwrap() {
            BufIO::Writer(w) => Some(w),
            _ => None,
        }
    }

    /// Returns a reference to the current `BufReaderWriter` read buffer data, if any.
    pub fn buffer(&self) -> Option<&[u8]> {
        self.buffer.as_ref().map(|b| &b[self.pos..])
    }

    /// Returns the buffer capacity of the underlying reader or writer.
    pub fn capacity(&self) -> usize {
        self.inner.as_ref().map_or(0, |b| b.capacity())
    }

    /// Low level function that indicates an amount of data has been consumed from the buffer and is not to be returned by the next read.  The buffer is dropped if all data has been consumed.
    pub fn consume(&mut self, amt: usize) {
        if let Some(b) = self.buffer.as_ref() {
            self.pos += amt;
            if self.pos >= b.len() {
                self.buffer = None
            }
        }
    }
}

impl<RW: Read + Write> Read for BufReaderWriterSeq<RW> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.as_mut().unwrap() {
            BufIO::Reader(r) => {
                if let Some(b) = &mut self.buffer {
                    let datalen = b.len() - self.pos;
                    let readlen = buf.len();
                    if datalen >= readlen {
                        buf.copy_from_slice(&b[self.pos..self.pos + readlen]);
                        if datalen > readlen {
                            self.pos += readlen;
                        } else {
                            self.buffer = None;
                        }
                        Ok(readlen)
                    } else {
                        buf[..datalen].copy_from_slice(&b[self.pos..self.pos + datalen]);
                        match r.read(&mut buf[datalen..]) {
                            Ok(n) => {
                                self.buffer = None;
                                Ok(datalen + n)
                            }
                            Err(e) => Err(e),
                        }
                    }
                } else {
                    r.read(buf)
                }
            }
            BufIO::Writer(w) => {
                w.flush()?;
                let rw = self.inner.take().unwrap().into_inner()?;
                self.inner = match self.capacity {
                    Some(c) => Some(BufIO::Reader(BufReader::with_capacity(c, rw))),
                    None => Some(BufIO::Reader(BufReader::new(rw))),
                };
                self.read(buf)
            }
        }
    }
}

impl<RW: Read + Write> Write for BufReaderWriterSeq<RW> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.as_mut().unwrap() {
            BufIO::Writer(w) => w.write(buf),
            BufIO::Reader(r) => {
                let rb = r.buffer();
                if !rb.is_empty() {
                    self.buffer = Some(Box::new(rb.to_vec()));
                    self.pos = 0;
                }
                let rw = self.inner.take().unwrap().into_inner()?;
                self.inner = match self.capacity {
                    Some(c) => Some(BufIO::Writer(BufWriter::with_capacity(c, rw))),
                    None => Some(BufIO::Writer(BufWriter::new(rw))),
                };
                self.write(buf)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.inner.as_mut() {
            Some(BufIO::Writer(w)) => Ok(w.flush()?),
            _ => Ok(()),
        }
    }
}
