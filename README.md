# BufReaderWriter
The `BufReaderWriter<RW>` is a convenience struct that facilitates automatic switching between buffered reading and writing from a single underlying Read + Write + Seek instance (generally applicable for  `std::fs::File`).  BufReaderWriter moves the underlying reader/writer between a BufReader and BufWriter as needed.  The reader/writer needs to be seekable as switching from reading to writing involves discarding the read buffer and seeking the underlying reader/writer back to the current position of the BufReader.

### Links

* Crate on [crates.io](https://crates.io/crates/bufreaderwriter)
* Documentation on [docs.rs](https://docs.rs/bufreaderwriter)
