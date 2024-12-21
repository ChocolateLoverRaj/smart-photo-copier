use std::{
    collections::HashMap,
    ffi::OsString,
    future,
    io::{self, ErrorKind},
    path::{Component, Path},
    sync::Arc,
    time::SystemTime,
};

use futures::{
    future::BoxFuture,
    stream::{self, BoxStream},
    FutureExt, StreamExt,
};
use tokio::sync::RwLock;

use crate::{
    async_fs::{AsyncFs, DirEntry, FileType, Metadata},
    fs::FS,
};

#[derive(Debug, Clone)]
pub enum FileOrFolder {
    File(Arc<RwLock<Vec<u8>>>),
    Folder(Arc<RwLock<HashMap<OsString, FileOrFolder>>>),
}

pub fn file(contents: Vec<u8>) -> FileOrFolder {
    FileOrFolder::File(Arc::new(RwLock::new(contents)))
}

pub fn empty_file() -> FileOrFolder {
    file(Default::default())
}

pub fn folder(folder: HashMap<OsString, FileOrFolder>) -> FileOrFolder {
    FileOrFolder::Folder(Arc::new(RwLock::new(folder)))
}

pub fn empty_folder() -> FileOrFolder {
    folder(Default::default())
}

#[derive(Debug, Clone)]
pub struct MockFs {
    root_file: FileOrFolder,
}

impl MockFs {
    pub fn new(root_file: FileOrFolder) -> Self {
        Self {
            root_file: root_file,
        }
    }
}

impl Default for MockFs {
    fn default() -> Self {
        Self::new(FileOrFolder::Folder(Default::default()))
    }
}

impl FS for MockFs {
    fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
        let from: &Path = from.as_ref();
        let to: &Path = to.as_ref();
        dbg!(from, to);
        Ok(0)
    }
}

impl AsyncFs for MockFs {
    fn read_dir(
        &self,
        path: impl AsRef<Path>,
    ) -> BoxFuture<'static, io::Result<BoxStream<'static, io::Result<Box<dyn DirEntry>>>>> {
        let path = path.as_ref().to_owned();
        let fs = self.clone();
        async move {
            let mut file_or_folder = None;
            for component in path.components() {
                match component {
                    Component::RootDir => match file_or_folder {
                        None => {
                            file_or_folder = Some(fs.root_file.clone());
                        }
                        Some(_) => {
                            Err(io::Error::new(
                                ErrorKind::NotFound,
                                "Root dir appeared twice in path".to_owned(),
                            ))?;
                        }
                    },
                    Component::ParentDir | Component::CurDir => {
                        Err(io::Error::new(
                            ErrorKind::NotFound,
                            "Relative paths not supported".to_owned(),
                        ))?;
                    }
                    Component::Prefix(_) => Err(io::Error::new(
                        ErrorKind::NotFound,
                        "Prefix not supported".to_owned(),
                    ))?,
                    Component::Normal(component) => match &file_or_folder {
                        None => Err(io::Error::new(
                            ErrorKind::NotFound,
                            "No root folder".to_owned(),
                        ))?,
                        Some(sub_file_or_folder) => match sub_file_or_folder.clone() {
                            FileOrFolder::File(_) => {
                                Err(io::Error::new(
                                    ErrorKind::NotADirectory,
                                    "What is sounds like".to_owned(),
                                ))?;
                            }
                            FileOrFolder::Folder(folder) => {
                                match folder.read().await.get(component) {
                                    Some(sub_item) => file_or_folder = Some(sub_item.to_owned()),
                                    None => {
                                        Err(io::Error::new(
                                            ErrorKind::NotFound,
                                            "Entry doesn't exist in folder".to_owned(),
                                        ))?;
                                    }
                                }
                            }
                        },
                    },
                }
            }
            match &file_or_folder {
                None => Err(io::Error::new(ErrorKind::NotFound, "".to_owned())),
                Some(file_or_folder) => match file_or_folder {
                    FileOrFolder::File(_) => {
                        Err(io::Error::new(ErrorKind::NotADirectory, "".to_owned()))
                    }
                    FileOrFolder::Folder(folder) => Ok(stream::iter(
                        folder.read().await.clone().into_iter().map(|(k, v)| {
                            Ok(Box::new(MockDirEntry {
                                file_or_folder: v.to_owned(),
                                name: k.to_owned(),
                            }) as Box<dyn DirEntry>)
                        }),
                    )
                    .boxed()),
                },
            }
        }
        .boxed()
    }
}

#[derive(Debug)]
struct MockDirEntry {
    name: OsString,
    file_or_folder: FileOrFolder,
}

impl DirEntry for MockDirEntry {
    fn file_type(&self) -> futures::future::BoxFuture<io::Result<FileType>> {
        let value = Ok(match self.file_or_folder {
            FileOrFolder::File(_) => FileType::File,
            FileOrFolder::Folder(_) => FileType::Dir,
        });
        future::ready(value).boxed()
    }

    fn file_name(&self) -> OsString {
        self.name.clone()
    }

    fn metadata(&self) -> BoxFuture<io::Result<Box<dyn Metadata>>> {
        let file_or_folder = self.file_or_folder.clone();
        async {
            Ok(Box::new(MockMetadata {
                len: match file_or_folder {
                    FileOrFolder::File(file) => file.read().await.len() as u64,
                    FileOrFolder::Folder(_) => 0,
                },
            }) as Box<_>)
        }
        .boxed()
    }
}

#[derive(Debug)]
struct MockMetadata {
    len: u64,
}

impl Metadata for MockMetadata {
    fn len(&self) -> u64 {
        self.len
    }

    fn modified(&self) -> io::Result<std::time::SystemTime> {
        // TODO: Mock last modified
        Ok(SystemTime::UNIX_EPOCH)
    }
}
