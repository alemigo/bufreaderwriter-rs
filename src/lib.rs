//! The `BufReaderWriter<RW>` is a convenience struct that facilitates automatic
//! switching between buffered reading and writing from a single underlying Read +
//! Write + Seek instance (generally applicable for  `std::fs::File`).  BufReaderWriter
//! moves the underlying instance between a BufReader and BufWriter as needed.
//!
//! The reader/writer needs to be seekable as switching from reading to writing
//! involves discarding the read buffer and seeking the underlying reader/writer back
//! to the current position of the BufReader.
//!
//! # Example
//!
//! ```no_run
//! # use std::io::{self, Read, Seek, SeekFrom, Write};
//! use bufreaderwriter::BufReaderWriter;
//! use tempfile::tempfile;
//!
//! fn main() -> io::Result<()> {
//!     let mut brw = BufReaderWriter::new_writer(tempfile()?);
//!     let data = "The quick brown fox jumps over the lazy dog".to_owned();
//!     brw.write(data.as_bytes())?;
//!
//!     brw.seek(SeekFrom::Start(0))?;
//!     let mut bin = vec![0; data.len()];
//!     brw.read(&mut bin)?;
//!     Ok(())
//! }
//! ```

use std::cell::Cell;
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

    fn into_inner(self) -> Result<RW, IntoInnerError<BufWriter<RW>>> {
        match self {
            BufIO::Reader(r) => Ok(r.into_inner()),
            BufIO::Writer(w) => Ok(w.into_inner()?),
        }
    }
}

pub struct BufReaderWriter<RW: Read + Write + Seek> {
    inner: Cell<Option<BufIO<RW>>>,
}

impl<RW: Read + Write + Seek> BufReaderWriter<RW> {
    /// Returns a new BufReaderWriter instance, expecting a write as the first operation.
    pub fn new_writer(rw: RW) -> BufReaderWriter<RW> {
        BufReaderWriter {
            inner: Cell::new(Some(BufIO::new_writer(rw))),
        }
    }

    /// Returns a new BufReaderWriter instance, expecting a read as the first operation.
    pub fn new_reader(rw: RW) -> BufReaderWriter<RW> {
        BufReaderWriter {
            inner: Cell::new(Some(BufIO::new_reader(rw))),
        }
    }

    /// Gets a mutable reference to the underlying reader/writer.
    pub fn get_mut(&mut self) -> &mut RW {
        self.inner.get_mut().as_mut().unwrap().get_mut()
    }

    /// Unwraps this `BufReaderWriter`, returning the underlying reader/writer.
    pub fn into_inner(self) -> Result<RW, IntoInnerError<BufWriter<RW>>> {
        self.inner.replace(None).unwrap().into_inner()
    }
}

impl<RW: Read + Write + Seek> Read for BufReaderWriter<RW> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.get_mut().as_mut().unwrap() {
            BufIO::Reader(r) => r.read(buf),
            BufIO::Writer(w) => {
                w.flush()?;
                self.inner.set(Some(BufIO::Reader(BufReader::new(
                    self.inner.replace(None).unwrap().into_inner()?,
                ))));
                self.read(buf)
            }
        }
    }
}

impl<RW: Read + Write + Seek> Write for BufReaderWriter<RW> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.get_mut().as_mut().unwrap() {
            BufIO::Writer(w) => w.write(buf),
            BufIO::Reader(r) => {
                r.seek(SeekFrom::Current(0))?;
                self.inner.set(Some(BufIO::Writer(BufWriter::new(
                    self.inner.replace(None).unwrap().into_inner()?,
                ))));
                self.write(buf)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.inner.get_mut() {
            Some(BufIO::Writer(w)) => Ok(w.flush()?),
            _ => Ok(()),
        }
    }
}

impl<RW: Read + Write + Seek> Seek for BufReaderWriter<RW> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match self.inner.get_mut().as_mut().unwrap() {
            BufIO::Writer(w) => w.seek(pos),
            BufIO::Reader(r) => r.seek(pos),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BufReaderWriter;
    use std::io::{Read, Seek, SeekFrom, Write};
    use tempfile::tempfile;

    #[test]
    fn test() {
        let file = tempfile().expect("Error creating temp file");
        let mut brw = BufReaderWriter::new_writer(file);
        let data = "The quick brown fox jumps over the lazy dog".to_owned();
        let data_len = data.len();

        for _ in 0..1000 {
            assert_eq!(data_len, brw.write(data.as_bytes()).expect("Write error"));
        }

        brw.seek(SeekFrom::Start(0)).expect("Seek error");
        for _ in 0..1000 {
            let mut bin = vec![0; data_len];
            let mut r = 0;
            while r < data_len {
                r += brw.read(&mut bin[r..]).expect("Read error");
            }
            assert_eq!(data.as_str(), std::str::from_utf8(&bin).unwrap());
        }

        brw.get_mut().set_len(3).expect("Error truncating file");
        brw.seek(SeekFrom::End(0)).expect("Seek error");
        brw.write(" dog".as_bytes()).expect("Write error");

        brw.seek(SeekFrom::Start(0)).expect("Seek error");
        let mut bin = vec![0; 7];
        assert_eq!(7, brw.read(&mut bin).expect("Read error"));
        assert_eq!("The dog".to_owned(), String::from_utf8(bin).unwrap());

        let _f = brw.into_inner().expect("Error extracting underlying file");
    }
}
