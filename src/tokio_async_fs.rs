use tokio::fs::read_dir;
use tokio_stream::wrappers::ReadDirStream;

use crate::async_fs::AsyncFs;

pub struct TokioAsyncFs {}

impl AsyncFs for TokioAsyncFs {
    async fn read_dir(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<impl futures::Stream<Item = std::io::Result<impl crate::async_fs::DirEntry>>>
    {
        read_dir(path)
            .await
            .map(|read_dir| ReadDirStream::new(read_dir))
    }
}
