use std::{io, path::Path};

pub trait FS {
    fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64>;
}
