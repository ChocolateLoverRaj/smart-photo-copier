use std::path::Path;

use crate::fs::FS;

pub async fn smart_copy<F>(_fs: F, from: &Path, to: &Path, check: &Path)
where
    F: FS,
{
}

#[cfg(test)]
mod tests {
    use crate::mock_fs::MockFS;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        smart_copy(MockFS::new(), "a".as_ref(), "b".as_ref(), "c".as_ref()).await;
    }
}
