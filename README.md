# BufReaderWriter
The `BufReaderWriter<RW>` is a convenience struct that facilitates automatic switching between buffered reading and writing from a single underlying Read + Write + Seek instance (generally designed for  `File`).  BufReaderWriter moves the underlying instance between a BufReader and BufWriter as needed.

### Links

* Crate on [crates.io](https://crates.io/crates/bufreaderwriter)
* Documentation on [docs.rs](https://docs.rs/bufreaderwriter)
