use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{
    cache,
    index::{Index, add_index},
};

pub fn save(
    index_base_path: &Path,
    cache_base_path: &Path,
    index: &mut (impl Index + Serialize),
    value: impl AsRef<[u8]>,
) -> anyhow::Result<PathBuf> {
    let cache_path = cache::put(cache_base_path, value)?;
    index.set_value_hash_path(cache_path);

    add_index(index_base_path, index)
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use serde::{Deserialize, Serialize};

    use crate::{index::Index, save};

    #[test]
    fn save_key_value_to_index_and_cache() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct Key {
            value_hash_path: Option<PathBuf>,
        }

        impl Index for Key {
            fn set_value_hash_path(&mut self, path: PathBuf) {
                self.value_hash_path = Some(path)
            }

            fn get_value_hash_path(&self) -> anyhow::Result<std::path::PathBuf> {
                unimplemented!()
            }
        }
        let mut key = Key {
            value_hash_path: None,
        };

        let key_path = save(
            Path::new("test_index"),
            Path::new("test_cache"),
            &mut key,
            b"some testing input",
        )
        .unwrap();

        let key_read = std::fs::read_to_string(key_path).unwrap();
        let key_read = serde_json::from_str::<Key>(&key_read).unwrap();

        eprintln!("{:?}", key_read);

        assert_eq!(key, key_read)
    }
}
