use std::time::SystemTime;
use std::{collections::HashMap, path::PathBuf};

use anyhow::Error;
use futures::{future::try_join_all, FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

use crate::async_fs::{AsyncFs, FileType};
use crate::read_dir_recursive::AsyncReadDirRecursiveExt;

pub struct SmartCopyOptions {
    pub sources: Vec<PathBuf>,
    pub destination: PathBuf,
    pub check: Vec<PathBuf>,
    pub save_metadata_in_sources: bool,
    pub save_metadata_in_check: bool,
    /// If the destination is inside a check folder, then the metadata will be saved in the root of the check folder and not the destination folder
    pub save_metadata_in_destination: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct FileSavedData {
    last_modified: (),
    hash: (),
}

type MetaData = HashMap<PathBuf, FileSavedData>;

#[derive(Debug, Clone)]
struct ExistingFile {
    path: PathBuf,
    len: u64,
    last_modified: SystemTime,
}

impl SmartCopyOptions {
    /// Make sure option are valid
    pub fn check(&self) -> Result<(), String> {
        // TODO: Actually check
        Ok(())
    }

    pub async fn smart_copy<T: AsyncFs + Clone + Send + 'static>(
        &self,
        fs: T,
    ) -> anyhow::Result<()> {
        self.check().unwrap();
        let metadata_file_name = ".smart_photo_copier";
        let existing_files = try_join_all(
            self.check
                .iter()
                .chain(
                    match self
                        .check
                        .iter()
                        .any(|path| self.destination.starts_with(path))
                    {
                        true => vec![],
                        false => vec![&self.destination],
                    },
                )
                .map(|folder_to_check| {
                    let fs = fs.clone();
                    async move {
                        let metadata_future = match self.save_metadata_in_sources {
                            true => Some(
                                async {
                                    let mut metadata = vec![];
                                    // TODO: Don't reat not found as an error
                                    // TODO: Different errors for different things
                                    // TODO: Maybe deserialize in a stream instead of reading everything into a vec first
                                    File::open(folder_to_check.join(metadata_file_name))
                                        .await
                                        .map_err(|_e| {})?
                                        .read_to_end(&mut metadata)
                                        .await
                                        .map_err(|_e| {})?;
                                    let metadata = postcard::from_bytes::<MetaData>(&metadata)
                                        .map_err(|_e| {})?;
                                    Ok::<_, ()>(metadata)
                                }
                                .shared(),
                            ),
                            false => None,
                        };
                        let vec = fs
                            .read_dir_recursive(folder_to_check)
                            .map(|(dir, entry)| entry.map(|entry| (dir, entry)))
                            .filter_map(|result| async move {
                                match result {
                                    Ok((dir, entry)) => match entry.file_type().await {
                                        Ok(file_type) => match file_type {
                                            FileType::File => Some({
                                                let path = dir.join(entry.file_name());
                                                entry
                                                    .metadata()
                                                    .await
                                                    .map(|metadata| ExistingFile {
                                                        path,
                                                        len: metadata.len(),
                                                        last_modified: metadata.modified().unwrap(),
                                                    })
                                                    .map_err(|e| e.to_string())
                                            }),
                                            FileType::Dir => None,
                                        },
                                        Err(e) => Some(Err(e.to_string())),
                                    },
                                    Err(e) => Some(Err(e.to_string())),
                                }
                            })
                            .try_collect::<Vec<_>>()
                            .await;
                        vec
                    }
                }),
        )
        .map_ok(|existing_files| existing_files.into_iter().flatten().collect::<Vec<_>>())
        .shared();
        try_join_all(self.sources.iter().map({
            let existing_files = existing_files.clone();
            move |source| {
                let fs = fs.clone();
                let existing_files = existing_files.clone();
                async move {
                    let metadata_future = match self.save_metadata_in_sources {
                        true => Some(
                            async {
                                let mut metadata = vec![];
                                // TODO: Don't reat not found as an error
                                // TODO: Different errors for different things
                                // TODO: Maybe deserialize in a stream instead of reading everything into a vec first
                                File::open(source.join(metadata_file_name))
                                    .await
                                    .map_err(|_e| {})?
                                    .read_to_end(&mut metadata)
                                    .await
                                    .map_err(|_e| {})?;
                                let metadata =
                                    postcard::from_bytes::<MetaData>(&metadata).map_err(|_e| {})?;
                                Ok::<_, ()>(metadata)
                            }
                            .shared(),
                        ),
                        false => None,
                    };
                    let results = fs
                        .read_dir_recursive(source)
                        .filter_map(|(dir, entry)| {
                            let existing_files = existing_files.clone();
                            async move {
                                match entry {
                                    Ok(entry) => match entry.file_type().await {
                                        Err(e) => Some(Err(e.to_string())),
                                        Ok(file_type) => match file_type {
                                            FileType::File => Some(match entry.metadata().await {
                                                Ok(metadata) => match existing_files.await {
                                                    Ok(existing_files) => {
                                                        let this_file_len = metadata.len();
                                                        let existing_file_with_same_len =
                                                            existing_files
                                                                .into_iter()
                                                                .filter(|existing_file| {
                                                                    existing_file.len
                                                                        == this_file_len
                                                                })
                                                                .collect::<Vec<_>>();
                                                        let path = dir.join(entry.file_name());
                                                        Ok((path, existing_file_with_same_len))
                                                    }
                                                    Err(e) => Err(e),
                                                },
                                                Err(e) => Err(e.to_string()),
                                            }),
                                            FileType::Dir => None,
                                        },
                                    },
                                    Err(e) => Some(Err(e.to_string())),
                                }
                            }
                        })
                        .collect::<Vec<_>>()
                        .await;
                    println!("Results: {results:?}");
                    Result::<_, Error>::Ok(results)
                }
            }
        }))
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::mock_fs::{empty_folder, file, folder, MockFs};

    use super::*;

    #[tokio::test]
    async fn it_works() {
        let fs = MockFs::new(folder({
            let mut m = HashMap::default();
            m.insert(
                "source".into(),
                folder({
                    let mut m = HashMap::default();
                    m.insert(
                        "First Day of School.png".into(),
                        file("Waiting for the school bus".into()),
                    );
                    m.insert(
                        "Birthday.png".into(),
                        file("Cutting the birthday cake".into()),
                    );
                    m
                }),
            );
            m.insert(
                "photos".into(),
                folder({
                    let mut m = HashMap::default();
                    m.insert(
                        "2023".into(),
                        folder({
                            let mut m = HashMap::default();
                            m.insert(
                                "First Day of School.png".into(),
                                file("Waiting for the school bus".into()),
                            );
                            m.insert(
                                "some random photo.jpg".into(),
                                file("WAITING FOR THE SCHOOL BUS".into()),
                            );
                            m
                        }),
                    );
                    m.insert("2024".into(), empty_folder());
                    m
                }),
            );
            m
        }));
        SmartCopyOptions {
            sources: vec!["/source".into()],
            destination: "/photos/2024".into(),
            check: vec!["/photos".into()],
            save_metadata_in_check: true,
            save_metadata_in_sources: true,
            save_metadata_in_destination: true,
        }
        .smart_copy(fs.clone())
        .await
        .unwrap();
    }
}
