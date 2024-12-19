use std::{collections::HashMap, path::PathBuf};

use anyhow::Error;
use futures::{
    future::{join_all, try_join_all},
    FutureExt, StreamExt, TryFuture, TryStreamExt,
};
use tokio::{fs::File, io::AsyncReadExt};

use crate::async_fs::AsyncFs;
use crate::read_dir_recursive::AsyncReadDirRecursiveExt;

pub struct SmartCopyOptions {
    sources: Vec<PathBuf>,
    destination: PathBuf,
    check: Vec<PathBuf>,
    save_metadata_in_sources: bool,
    save_metadata_in_check: bool,
}

type MetaData = HashMap<PathBuf, bool>;

impl SmartCopyOptions {
    async fn smart_copy<T: AsyncFs + Clone + Send + 'static>(&self, fs: T) -> anyhow::Result<()> {
        let metadata_file_name = ".smart_photo_copier";
        try_join_all(self.sources.iter().map(|source| {
            let fs = fs.clone();
            async move {
                let metadata_future = async {
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
                    let metadata = postcard::from_bytes::<MetaData>(&metadata).map_err(|_e| {})?;
                    Ok::<_, ()>(metadata)
                }
                .shared();
                let results = fs.read_dir_recursive(source).collect::<Vec<_>>().await;
                println!("Results: {results:?}");
                Result::<_, Error>::Ok(results)
            }
        }))
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::mock_fs::{empty_file, folder, MockFs};

    use super::*;

    #[tokio::test]
    async fn it_works() {
        SmartCopyOptions {
            sources: vec!["/source".into()],
            destination: "/photos/2024".into(),
            check: vec!["/photos".into()],
            save_metadata_in_check: true,
            save_metadata_in_sources: true,
        }
        .smart_copy(MockFs::new(folder({
            let mut m = HashMap::default();
            m.insert(
                "source".into(),
                folder({
                    let mut m = HashMap::default();
                    m.insert("First Day of School.png".into(), empty_file());
                    m.insert("Birthday.png".into(), empty_file());
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
                            m.insert("First Day of School.png".into(), empty_file());
                            m
                        }),
                    );
                    m.insert("2024".into(), folder(Default::default()));
                    m
                }),
            );
            m
        })))
        .await
        .unwrap();
    }
}
