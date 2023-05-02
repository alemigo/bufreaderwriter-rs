//! The `BufReaderWriterRand<RW>` and `BufReaderWriterSeq<RW>` are convenience structs that facilitate automatic
//! switching between buffered reading and writing from a single underlying IO instance. `BufReaderWriterRand` is
//! for random access IO (i.e. Read + Write + Seek, such as `std::fs::File`), while `BufReaderWriterSeq` is for sequential
//! IO (i.e. Read + Write, such as `std::net::TcpStream`).
//!
//! Both structs move the underlying IO instance between a BufReader and BufWriter as needed.  However, when switching from
//! reading to writing, `BufReaderWriterRand` discards any buffered data and seeks the underlying IO instance back to the
//! current BufReader position, while `BufReaderWriterSeq` saves any buffered data and makes it available for subsequent
//! reads.
//!
//! # Example
//!
//! ```no_run
//! # use std::io::{self, Read, Seek, SeekFrom, Write};
//! use bufreaderwriter::rand::BufReaderWriterRand;
//! use tempfile::tempfile;
//!
//! fn main() -> io::Result<()> {
//!     let mut brw = BufReaderWriterRand::new_writer(tempfile()?);
//!     let data = "The quick brown fox jumps over the lazy dog".to_owned();
//!     brw.write(data.as_bytes())?;
//!
//!     brw.seek(SeekFrom::Start(0))?;
//!     let mut bin = vec![0; data.len()];
//!     brw.read(&mut bin)?;
//!     Ok(())
//! }
//! ```

pub mod rand;
pub mod seq;

#[cfg(test)]
mod tests {
    use crate::rand::BufReaderWriterRand;
    use crate::seq::BufReaderWriterSeq;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::Duration;
    use tempfile::tempfile;

    #[test]
    fn testrand() {
        let file = tempfile().expect("Error creating temp file");
        let mut brw = BufReaderWriterRand::new_writer(file);
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

    #[test]
    fn testseq() {
        let data = "The quick brown fox jumps over the lazy dog".to_owned();
        let data_len = data.len();

        let handle = thread::spawn(|| {
            let tcp = TcpListener::bind("127.0.0.1:8080").expect("TcpListener error");
            match tcp.accept() {
                Ok((mut socket, _addr)) => {
                    socket
                        .set_read_timeout(Some(Duration::new(2, 0)))
                        .expect("Read timeout");
                    let mut buf = vec![0_u8; 100];
                    loop {
                        match socket.read(&mut buf[..]) {
                            Ok(n) => {
                                socket.write(&buf[0..n]).expect("write io error");
                            }
                            Err(e) => match e.kind() {
                                std::io::ErrorKind::TimedOut => break,
                                _ => panic!("listener read error {}", e),
                            },
                        }
                    }
                }
                Err(e) => panic!("TCP Listen error {}", e),
            }
        });

        let socket2 = TcpStream::connect("127.0.0.1:8080").expect("TcpStream error");
        let mut brw = BufReaderWriterSeq::new_writer(socket2);

        thread::sleep(Duration::new(1, 0));
        assert_eq!(data_len, brw.write(data.as_bytes()).expect("Write error"));

        let mut buf = vec![0_u8; 10];
        let _n = brw.read(&mut buf[..]).expect("read io error");
        assert_eq!(std::str::from_utf8(&buf).unwrap(), &data[0..10]);

        let _n = brw.write(data.as_bytes()).expect("write io error");
        let _n = brw.write(data.as_bytes()).expect("write io error");

        let mut buf = vec![0_u8; 5];
        let _n = brw.read(&mut buf[..]).expect("read io error");
        let outdata = std::str::from_utf8(&buf).unwrap();
        assert_eq!(outdata, &data[10..15]);

        let mut buf = vec![0_u8; (2 * data_len) - 15];
        let _n = brw.read(&mut buf[..]).expect("read io error");
        let outdata = std::str::from_utf8(&buf).unwrap();
        assert_eq!(&outdata[0..data_len - 15], &data[15..]);
        assert_eq!(&outdata[data_len - 15..], &data);

        handle.join().expect("Join thread error");
    }
}
