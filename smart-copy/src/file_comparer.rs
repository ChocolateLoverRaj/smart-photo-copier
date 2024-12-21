use std::{collections::HashMap, path::PathBuf, time::SystemTime};

use blake3::Hash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
struct FileSavedData {
    last_modified: SystemTime,
    /// The hash of the first 4096 bytes
    small_hash: Hash,
    /// The hash of the entire file
    big_hash: Option<Hash>,
}

type MetaData = HashMap<PathBuf, FileSavedData>;

struct FileComparer {
    /// The root folders which contain a file storing the hashes for future use
    root_folders: Vec<PathBuf>,
}

impl FileComparer {
    pub fn new(root_folders: Vec<PathBuf>) -> Self {
        Self { root_folders }
    }
}
