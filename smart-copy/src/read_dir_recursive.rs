use std::{
    io,
    path::{Path, PathBuf},
};

use futures::{
    future,
    stream::{self, BoxStream},
    FutureExt, StreamExt,
};

use crate::async_fs::{AsyncFs, DirEntry, FileType};

// pub trait AsyncReadDirRecursiveExt {
//     fn read_dir_recursive(
//         &self,
//         path: impl AsRef<Path>,
//     ) -> impl Stream<Item = (PathBuf, io::Result<Box<dyn DirEntry>>)> + Send + 'static;
// }

// impl<T: AsyncFs + Clone + Send + 'static> AsyncReadDirRecursiveExt for T {
//     fn read_dir_recursive(
//         &self,
//         path: impl AsRef<Path>,
//     ) -> impl Stream<Item = (PathBuf, io::Result<Box<dyn DirEntry>>)> + Send + 'static {
//         fn read_dir_recursive<T: AsyncFs + Clone + Send + 'static>(
//             fs: T,
//             path: PathBuf,
//         ) -> impl Future<
//             Output = Vec<RecursiveItem<'static, (PathBuf, io::Result<Box<dyn DirEntry>>)>>,
//         > + Send
//                + 'static {
//             let fs = fs.clone();
//             let path = path.clone();
//             async move {
//                 match fs.read_dir(&path).await {
//                     Ok(dir) => {
//                         dir.filter_map({
//                             let fs = fs.clone();
//                             let path = path.clone();
//                             move |entry| {
//                                 let fs = fs.clone();
//                                 let path = path.clone();
//                                 async move {
//                                     match entry {
//                                         Ok(entry) => match entry.file_type().await {
//                                             Ok(file_type) => match file_type {
//                                                 FileType::Dir => Some(RecursiveItem::Future(
//                                                     read_dir_recursive(
//                                                         fs.clone(),
//                                                         path.join(entry.file_name()),
//                                                     )
//                                                     .boxed(),
//                                                 )),
//                                                 FileType::File => {
//                                                     Some(RecursiveItem::Item((path, Ok(entry))))
//                                                 }
//                                             },
//                                             Err(e) => Some(RecursiveItem::Item((path, Err(e)))),
//                                         },
//                                         Err(e) => Some(RecursiveItem::Item((path, Err(e)))),
//                                     }
//                                 }
//                             }
//                         })
//                         .collect::<Vec<_>>()
//                         .await
//                     }
//                     Err(err) => {
//                         vec![RecursiveItem::Item((path, Err(err)))]
//                     }
//                 }
//             }
//         }
//         RecursiveStream2::new(read_dir_recursive(self.clone(), path.as_ref().to_path_buf()).boxed())
//             .flat_map_unordered(None, |vec| stream::iter(vec))
//     }
// }

pub trait AsyncReadDirRecursiveExt {
    fn read_dir_recursive(
        &self,
        path: impl AsRef<Path>,
    ) -> BoxStream<'static, (PathBuf, io::Result<Box<dyn DirEntry>>)>;
}

impl<T: AsyncFs + Clone + Send + 'static> AsyncReadDirRecursiveExt for T {
    fn read_dir_recursive(
        &self,
        path: impl AsRef<Path>,
    ) -> BoxStream<'static, (PathBuf, io::Result<Box<dyn DirEntry>>)> {
        let path = path.as_ref().to_owned();
        let fs = self.clone();
        self.read_dir(path.clone())
            .into_stream()
            .flat_map_unordered(None, {
                let path = path.clone();
                let fs = fs.clone();
                move |result| match result {
                    Ok(stream) => stream
                        .flat_map_unordered(None, {
                            let path = path.clone();
                            let fs = fs.clone();
                            move |result| match result {
                                Ok(entry) => async { (entry.file_type().await, entry) }
                                    .into_stream()
                                    .flat_map({
                                        let fs = fs.clone();
                                        let path = path.clone();
                                        move |(result, entry)| match result {
                                            Ok(file_type) => match file_type {
                                                FileType::Dir => {
                                                    let file_name = entry.file_name();
                                                    stream::once(future::ready((
                                                        path.clone(),
                                                        Ok(entry),
                                                    )))
                                                    .chain(
                                                        fs.read_dir_recursive(path.join(file_name)),
                                                    )
                                                    .boxed()
                                                }
                                                FileType::File => stream::once(future::ready((
                                                    path.clone(),
                                                    Ok(entry),
                                                )))
                                                .boxed(),
                                            },
                                            Err(e) => {
                                                stream::once(future::ready((path.clone(), Err(e))))
                                                    .boxed()
                                            }
                                        }
                                    })
                                    .boxed(),
                                Err(e) => {
                                    stream::once(future::ready((path.clone(), Err(e)))).boxed()
                                }
                            }
                        })
                        .boxed(),
                    Err(e) => stream::once(future::ready((path.clone(), Err(e)))).boxed(),
                }
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use futures::StreamExt;

    use super::AsyncReadDirRecursiveExt;
    use crate::{
        async_fs::FileType,
        mock_fs::{empty_file, folder, MockFs},
    };

    #[tokio::test]
    async fn mock_fs() {
        let mut vec = MockFs::new(folder({
            let mut m = HashMap::default();
            m.insert("file.txt".into(), empty_file());
            m.insert(
                "folder".into(),
                folder({
                    let mut m = HashMap::default();
                    m.insert("cat.png".into(), empty_file());
                    m.insert("dog.png".into(), empty_file());
                    m
                }),
            );
            m
        }))
        .read_dir_recursive("/")
        .filter_map(|(path, result)| async move {
            let entry = result.unwrap();
            Some((path, entry.file_name(), entry.file_type().await.unwrap()))
        })
        .collect::<Vec<_>>()
        .await;
        vec.sort();
        assert_eq!(
            vec,
            vec![
                ("/".into(), "file.txt".into(), FileType::File),
                ("/".into(), "folder".into(), FileType::Dir),
                ("/folder".into(), "cat.png".into(), FileType::File),
                ("/folder".into(), "dog.png".into(), FileType::File),
            ]
        )
    }
}
