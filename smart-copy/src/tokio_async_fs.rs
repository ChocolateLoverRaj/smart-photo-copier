use futures::StreamExt;
use tokio::fs::read_dir;
use tokio_stream::wrappers::ReadDirStream;

use crate::async_fs::{AsyncFs, DirEntry};

pub struct TokioAsyncFs {}

impl AsyncFs for TokioAsyncFs {
    fn read_dir(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> impl std::future::Future<
        Output = std::io::Result<
            impl futures::Stream<Item = std::io::Result<Box<dyn crate::async_fs::DirEntry>>>
                + Send
                + 'static,
        >,
    > + Send {
        let path = path.as_ref().to_owned();
        async {
            read_dir(path).await.map(|read_dir| {
                ReadDirStream::new(read_dir).map(|dir_entry| {
                    dir_entry.map(|dir_entry| Box::new(dir_entry) as Box<dyn DirEntry>)
                })
            })
        }
    }
}
