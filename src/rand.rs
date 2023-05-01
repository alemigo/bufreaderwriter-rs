use std::io::{self, BufReader, BufWriter, IntoInnerError, Read, Seek, SeekFrom, Write};

enum BufIO<RW: Read + Write + Seek> {
    Reader(BufReader<RW>),
    Writer(BufWriter<RW>),
}

impl<RW: Read + Write + Seek> BufIO<RW> {
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

pub struct BufReaderWriterRand<RW: Read + Write + Seek> {
    inner: Option<BufIO<RW>>,
}

impl<RW: Read + Write + Seek> BufReaderWriterRand<RW> {
    /// Returns a new BufReaderWriterRand instance, expecting a write as the first operation.
    pub fn new_writer(rw: RW) -> BufReaderWriterRand<RW> {
        BufReaderWriterRand {
            inner: Some(BufIO::new_writer(rw)),
        }
    }

    /// Returns a new BufReaderWriter instance, expecting a read as the first operation.
    pub fn new_reader(rw: RW) -> BufReaderWriterRand<RW> {
        BufReaderWriterRand {
            inner: Some(BufIO::new_reader(rw)),
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

impl<RW: Read + Write + Seek> Read for BufReaderWriterRand<RW> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.as_mut().unwrap() {
            BufIO::Reader(r) => r.read(buf),
            BufIO::Writer(w) => {
                w.flush()?;
                let rw = self.inner.take().unwrap().into_inner()?;
                self.inner = Some(BufIO::Reader(BufReader::new(rw)));
                self.read(buf)
            }
        }
    }
}

impl<RW: Read + Write + Seek> Write for BufReaderWriterRand<RW> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.as_mut().unwrap() {
            BufIO::Writer(w) => w.write(buf),
            BufIO::Reader(r) => {
                r.seek(SeekFrom::Current(0))?;
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

impl<RW: Read + Write + Seek> Seek for BufReaderWriterRand<RW> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match self.inner.as_mut().unwrap() {
            BufIO::Writer(w) => w.seek(pos),
            BufIO::Reader(r) => r.seek(pos),
        }
    }
}
