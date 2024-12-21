use futures::{FutureExt, StreamExt};
use tokio::fs::read_dir;
use tokio_stream::wrappers::ReadDirStream;

use crate::async_fs::{AsyncFs, DirEntry};

pub struct TokioAsyncFs {}

impl AsyncFs for TokioAsyncFs {
    fn read_dir(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> futures::future::BoxFuture<
        'static,
        std::io::Result<futures::stream::BoxStream<'static, std::io::Result<Box<dyn DirEntry>>>>,
    > {
        let path = path.as_ref().to_owned();
        async {
            read_dir(path).await.map(|read_dir| {
                ReadDirStream::new(read_dir)
                    .map(|dir_entry| {
                        dir_entry.map(|dir_entry| Box::new(dir_entry) as Box<dyn DirEntry>)
                    })
                    .boxed()
            })
        }
        .boxed()
    }
}
