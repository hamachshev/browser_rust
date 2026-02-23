use std::{
    fs::DirBuilder,
    path::{Path, PathBuf},
};

use anyhow::Context;
use hex;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn add_index(
    index_base_path: &Path,
    index: &mut (impl Serialize + Index),
    cache_path: PathBuf,
) -> anyhow::Result<PathBuf> {
    let index_path = get_key_path(index_base_path, index)?;
    let parent = index_path
        .parent()
        .context("couldnt get parent in 'key' path")?;
    DirBuilder::new().recursive(true).create(parent)?;

    // need to do this after hashing for the index path
    // becasue when retrieve will hash the non-cache-path-ed key because we are looking to retrieve
    // the path to the cached item which we wont have. So add the path now, reserialize, and then
    // write key to index

    index.set_value_hash_path(cache_path);
    let serialized = serde_json::to_string(index)?;
    std::fs::write(&index_path, serialized)?;
    Ok(index_path)
}
pub(crate) fn get_key_path(
    index_base_path: &Path,
    index: &(impl Serialize + Index),
) -> anyhow::Result<PathBuf> {
    let mut index_path = PathBuf::from(index_base_path);
    let serialized = serde_json::to_string(index)?;
    let hash: String = hex::encode(Sha256::digest(&serialized).to_vec());
    index_path.push(&hash[0..2]);
    index_path.push(&hash[2..4]);
    index_path.push(&hash[4..]);

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
        struct Key {
            data: String,
            inner_hash_path: Option<PathBuf>,
        }

        let mut test_index = Key {
            data: "test_add_index".to_string(),
            inner_hash_path: None,
        };
        impl Index for Key {
            fn set_value_hash_path(&mut self, path: PathBuf) {
                self.inner_hash_path = Some(path);
            }

            fn get_value_hash_path(&self) -> anyhow::Result<PathBuf> {
                unimplemented!()
            }
        }

        let path = add_index(
            index_path.as_path(),
            &mut test_index,
            "this is a fake path".into(),
        )
        .unwrap();

        let contents = std::fs::read_to_string(path).unwrap();

        assert_eq!(contents, serde_json::to_string(&test_index).unwrap());
    }
}
