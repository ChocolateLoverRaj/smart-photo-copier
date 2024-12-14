use std::path::PathBuf;

use crate::async_fs::AsyncFs;

pub struct SmartCopyOptions {
    source: PathBuf,
    destination: PathBuf,
    check: PathBuf,
}

impl SmartCopyOptions {
    async fn smart_copy(&self, fs: &impl AsyncFs) {
        fs.read_dir(&self.check).await;
    }
}

#[cfg(test)]
mod tests {
    use crate::mock_fs::MockFs;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        SmartCopyOptions {
            source: "a".into(),
            destination: "b".into(),
            check: "c".into(),
        }
        .smart_copy(&MockFs::new())
        .await;
    }
}
