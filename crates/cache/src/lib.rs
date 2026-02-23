mod cache;
mod index;

use std::path::PathBuf;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub use crate::index::Index;
use crate::index::{add_index, get_key_path};

pub struct Cache {
    index_path: PathBuf,
    cache_path: PathBuf,
}

impl Cache {
    pub fn new(index_path: PathBuf, cache_path: PathBuf) -> Self {
        Self {
            index_path,
            cache_path,
        }
    }
    pub fn save(
        &self,
        key: &mut (impl Serialize + Index),
        value: impl AsRef<[u8]>,
    ) -> anyhow::Result<PathBuf> {
        let cache_path = cache::put(&self.cache_path, value)?;

        add_index(&self.index_path, key, cache_path)
    }
    pub fn get<T>(&self, key: &T) -> anyhow::Result<Vec<u8>>
    where
        T: Serialize + Index + DeserializeOwned,
    {
        let key_path = get_key_path(&self.index_path, key)?;

        let key = std::fs::read_to_string(key_path)?;
        let contents: T = serde_json::from_str(&key)?;

        let cache_path = contents.get_value_hash_path()?;
        Ok(std::fs::read(cache_path)?)
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use anyhow::Context;
    use serde::{Deserialize, Serialize};

    use crate::{Cache, Index};

    #[test]
    fn save_in_cache() {
        let cache = Cache::new("test-index".into(), "test-cache".into());

        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Key {
            inner_hash_path: Option<PathBuf>,
        }

        impl Index for Key {
            fn set_value_hash_path(&mut self, path: PathBuf) {
                self.inner_hash_path = Some(path);
            }

            fn get_value_hash_path(&self) -> anyhow::Result<PathBuf> {
                unimplemented!()
            }
        }
        let mut key = Key {
            inner_hash_path: None,
        };

        let key_path = cache.save(&mut key, "hello").unwrap();

        let key_raw = std::fs::read_to_string(&key_path).unwrap();

        let key_deser: Key = serde_json::from_str(&key_raw).unwrap();

        assert_eq!(key_deser, key);

        let Some(value_path) = key_deser.inner_hash_path else {
            panic!("missing value path in deser")
        };

        let value = std::fs::read_to_string(value_path).unwrap();

        assert_eq!("hello", value);
    }
    #[test]
    fn retrieve_from_cache() {
        let cache = Cache::new("test-index".into(), "test-cache".into());

        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Key {
            data: String,
            inner_hash_path: Option<PathBuf>,
        }

        impl Index for Key {
            fn set_value_hash_path(&mut self, path: PathBuf) {
                self.inner_hash_path = Some(path);
            }

            fn get_value_hash_path(&self) -> anyhow::Result<PathBuf> {
                self.inner_hash_path
                    .clone()
                    .context("missing inner hash path")
            }
        }
        let mut key = Key {
            data: "retrieve_from_cache".into(),
            inner_hash_path: None,
        };

        let _ = cache.save(&mut key, "hello from the other side").unwrap();
        let new_key = Key {
            data: "retrieve_from_cache".into(),
            inner_hash_path: None,
        };

        let res = cache.get(&new_key).unwrap();

        assert_eq!(res, b"hello from the other side")
    }
}
