# BufReaderWriter
The `BufReaderWriterRand<RW>` and `BufReaderWriterSeq<RW>` are convenience structs that facilitate automatic
switching between buffered reading and writing from a single underlying IO instance. `BufReaderWriterRand` is
for random access IO (i.e. Read + Write + Seek, such as `std::fs::File`), while `BufReaderWriterSeq` is for sequential
IO (i.e. Read + Write, such as `std::net::TcpStream`).

Both structs move the underlying IO instance between a BufReader and BufWriter as needed.  However, when switching from
reading to writing, `BufReaderWriterRand` discards any buffered data and seeks the underlying IO instance back to the
current BufReader position, while `BufReaderWriterSeq` saves any buffered data and makes it available for subsequent
reads.

### Links

* Crate on [crates.io](https://crates.io/crates/bufreaderwriter)
* Documentation on [docs.rs](https://docs.rs/bufreaderwriter)
