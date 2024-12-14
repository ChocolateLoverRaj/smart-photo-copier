use std::{
    ffi::OsString,
    io::{self, ErrorKind},
    path::{Component, Path, PathBuf},
};

use dashmap::DashMap;
use futures::stream;

use crate::{async_fs::AsyncFs, fs::FS};

enum FileOrFolder {
    File,
    Folder(DashMap<OsString, FileOrFolder>),
}

pub struct MockFs {
    root_folder: FileOrFolder,
}

impl MockFs {
    pub fn new() -> Self {
        MockFs {
            root_folder: FileOrFolder::Folder(DashMap::new()),
        }
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
    async fn read_dir(
        &self,
        path: impl AsRef<Path>,
    ) -> io::Result<impl futures::Stream<Item = io::Result<impl crate::async_fs::DirEntry>>> {
        let mut file_or_folder = None;
        for component in path.as_ref().components() {
            match component {
                Component::RootDir => match file_or_folder {
                    None => {
                        file_or_folder = Some(self.root_folder);
                    }
                    Some(_) => {
                        Err("Two root paths!".into())?;
                    }
                },
                Component::ParentDir | Component::CurDir => {
                    Err("Relative paths not supported".into())?
                }
                Component::Prefix(_) => Err("Prefix not supported.".into())?,
                Component::Normal(component) => match file_or_folder {
                    None => Err(io::Error::new(ErrorKind::NotFound, "".into()))?,
                    Some(FileOrFolder::File) => {
                        Err(io::Error::new(ErrorKind::NotADirectory, "".into()))?;
                    }
                    Some(FileOrFolder::Folder(folder)) => folder.get(key),
                },
            }
        }
        match file_or_folder {}
    }
}
