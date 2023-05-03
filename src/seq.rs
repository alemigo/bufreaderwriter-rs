use std::io::{self, BufReader, BufWriter, IntoInnerError, Read, Write};

enum BufIO<RW: Read + Write> {
    Reader(BufReader<RW>),
    Writer(BufWriter<RW>),
}

impl<RW: Read + Write> BufIO<RW> {
    fn new_writer(rw: RW) -> BufIO<RW> {
        BufIO::Writer(BufWriter::new(rw))
    }

    fn new_reader(rw: RW) -> BufIO<RW> {
        BufIO::Reader(BufReader::new(rw))
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
}

pub struct BufReaderWriterSeq<RW: Read + Write> {
    inner: Option<BufIO<RW>>,
    buffer: Box<Vec<u8>>,
    pos: usize,
}

impl<RW: Read + Write> BufReaderWriterSeq<RW> {
    /// Returns a new BufReaderWriterSeq instance, expecting a write as the first operation.
    pub fn new_writer(rw: RW) -> BufReaderWriterSeq<RW> {
        BufReaderWriterSeq {
            inner: Some(BufIO::new_writer(rw)),
            buffer: Box::new(vec![]),
            pos: 0,
        }
    }

    /// Returns a new BufReaderWriter instance, expecting a read as the first operation.
    pub fn new_reader(rw: RW) -> BufReaderWriterSeq<RW> {
        BufReaderWriterSeq {
            inner: Some(BufIO::new_reader(rw)),
            buffer: Box::new(vec![]),
            pos: 0,
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

    /// Unwraps this `BufReaderWriter`, returning the underlying reader/writer.
    pub fn into_inner(self) -> Result<RW, IntoInnerError<BufWriter<RW>>> {
        self.inner.unwrap().into_inner()
    }
}

impl<RW: Read + Write> Read for BufReaderWriterSeq<RW> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.as_mut().unwrap() {
            BufIO::Reader(r) => {
                if self.pos >= self.buffer.len() {
                    r.read(buf)
                } else {
                    let datalen = self.buffer.len() - self.pos;
                    let readlen = buf.len();
                    if datalen >= readlen {
                        buf.copy_from_slice(&self.buffer[self.pos..self.pos + readlen]);
                        self.pos += readlen;
                        Ok(readlen)
                    } else {
                        self.buffer.resize(self.pos + readlen, 0);
                        match r.read(&mut self.buffer[self.pos + datalen..self.pos + readlen]) {
                            Ok(n) => {
                                buf.copy_from_slice(&self.buffer[self.pos..self.pos + readlen]);
                                self.buffer.truncate(0);
                                self.pos = 0;
                                Ok(datalen + n)
                            }
                            Err(e) => Err(e),
                        }
                    }
                }
            }
            BufIO::Writer(w) => {
                w.flush()?;
                let rw = self.inner.take().unwrap().into_inner()?;
                self.inner = Some(BufIO::Reader(BufReader::new(rw)));
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
                self.buffer = Box::new(r.buffer().to_vec());
                self.pos = 0;
                let rw = self.inner.take().unwrap().into_inner()?;
                self.inner = Some(BufIO::Writer(BufWriter::new(rw)));
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
