use std::{ffi::OsString, fmt::Debug, future::Future, io, path::Path};

use futures::{future::BoxFuture, FutureExt, Stream};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileType {
    File,
    Dir,
}

impl From<std::fs::FileType> for FileType {
    fn from(value: std::fs::FileType) -> Self {
        if value.is_file() {
            Self::File
        } else if value.is_dir() {
            Self::Dir
        } else {
            unimplemented!()
        }
    }
}

pub trait DirEntry: Debug + Send + Sync {
    fn file_type(&self) -> BoxFuture<io::Result<FileType>>;
    fn file_name(&self) -> OsString;
}

impl DirEntry for tokio::fs::DirEntry {
    fn file_type(&self) -> BoxFuture<io::Result<FileType>> {
        async { self.file_type().await.map(|file_type| file_type.into()) }.boxed()
    }

    fn file_name(&self) -> OsString {
        self.file_name()
    }
}

pub trait AsyncFs {
    fn read_dir(
        &self,
        path: impl AsRef<Path>,
    ) -> impl Future<
        Output = io::Result<impl Stream<Item = io::Result<Box<dyn DirEntry>>> + Send + 'static>,
    > + Send;
}
