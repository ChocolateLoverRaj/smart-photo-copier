use std::{fs::FileType, io, path::Path};

use futures::Stream;

pub trait DirEntry {
    async fn file_type(&self) -> io::Result<FileType>;
}

impl DirEntry for tokio::fs::DirEntry {
    async fn file_type(&self) -> io::Result<FileType> {
        self.file_type().await
    }
}

pub trait AsyncFs {
    async fn read_dir(
        &self,
        path: impl AsRef<Path>,
    ) -> io::Result<impl Stream<Item = io::Result<impl DirEntry>>>;
}
