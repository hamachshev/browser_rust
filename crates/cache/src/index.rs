use std::{
    fs::DirBuilder,
    path::{Path, PathBuf},
};

use hex;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn add_index(index_path: &Path, index: &impl Serialize) -> anyhow::Result<PathBuf> {
    let mut index_path = PathBuf::from(index_path);
    let serialized = serde_json::to_string(index)?;
    let hash: String = hex::encode(Sha256::digest(&serialized).to_vec());
    index_path.push(&hash[0..2]);
    index_path.push(&hash[2..4]);

    DirBuilder::new().recursive(true).create(&index_path)?;
    index_path.push(&hash[4..]);

    std::fs::write(&index_path, serialized)?;
    Ok(index_path)
}

pub trait Index {
    fn set_value_hash_path(&mut self, path: PathBuf);
    fn get_value_hash_path(&self) -> anyhow::Result<PathBuf>;
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn add_index_test() {
        let index_path = PathBuf::from("test-index");

        #[derive(Serialize, Deserialize)]
        struct Index {
            inner_hash: String,
        }

        let test_index = Index {
            inner_hash: "Nowhere".to_string(),
        };

        let path = add_index(&index_path, &test_index).unwrap();

        let contents = std::fs::read_to_string(path).unwrap();

        assert_eq!(contents, serde_json::to_string(&test_index).unwrap());
    }
}
