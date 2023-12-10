use std::{io, path::Path};

use crate::fs::FS;

pub struct MockFS {}

impl FS for MockFS {
    fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
        let from: &Path = from.as_ref();
        let to: &Path = to.as_ref();
        dbg!(from, to);
        Ok(0)
    }
}

impl MockFS {
    pub fn new() -> Self {
        MockFS {}
    }
}
