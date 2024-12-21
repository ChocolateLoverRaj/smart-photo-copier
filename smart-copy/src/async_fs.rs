use std::{ffi::OsString, fmt::Debug, io, path::Path, time::SystemTime};

use futures::{future::BoxFuture, stream::BoxStream, FutureExt};

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

pub trait Metadata: Debug {
    fn len(&self) -> u64;
    fn modified(&self) -> io::Result<SystemTime>;
}

impl Metadata for std::fs::Metadata {
    fn len(&self) -> u64 {
        self.len()
    }

    fn modified(&self) -> io::Result<SystemTime> {
        self.modified()
    }
}

pub trait DirEntry: Debug + Send + Sync {
    fn file_type(&self) -> BoxFuture<io::Result<FileType>>;
    fn file_name(&self) -> OsString;
    fn metadata(&self) -> BoxFuture<io::Result<Box<dyn Metadata>>>;
}

impl DirEntry for tokio::fs::DirEntry {
    fn file_type(&self) -> BoxFuture<io::Result<FileType>> {
        async { self.file_type().await.map(|file_type| file_type.into()) }.boxed()
    }

    fn file_name(&self) -> OsString {
        self.file_name()
    }

    fn metadata(&self) -> BoxFuture<io::Result<Box<dyn Metadata>>> {
        async {
            self.metadata()
                .await
                .map(|metadata| Box::new(metadata) as Box<dyn Metadata>)
        }
        .boxed()
    }
}

pub trait AsyncFs {
    fn read_dir(
        &self,
        path: impl AsRef<Path>,
    ) -> BoxFuture<'static, io::Result<BoxStream<'static, io::Result<Box<dyn DirEntry>>>>>;
}
