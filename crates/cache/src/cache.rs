use std::{
    fs::DirBuilder,
    path::{Path, PathBuf},
};

use anyhow::Context;
use ssri::Integrity;

pub struct Cacher;

pub fn put(cache: impl AsRef<Path>, content: &[u8]) -> anyhow::Result<()> {
    let integrity = Integrity::from(content);
    let path = get_path(cache.as_ref(), &integrity)?;
    let (dirs, _) = path
        .to_str()
        .context("could not convert path to str")?
        .rsplit_once('/')
        .context("could not find dir structure in integrity path")?;
    DirBuilder::new().recursive(true).create(dirs)?;
    std::fs::write(path, content)?;
    Ok(())
}

fn get_path(cache: impl AsRef<Path>, integrity: &Integrity) -> anyhow::Result<PathBuf> {
    let mut path = PathBuf::from(cache.as_ref());
    let integrity_string = integrity.to_string();
    let (algo, rest) = integrity_string
        .split_once('-')
        .context("missing '-' in hash in identity string")?;
    path.push(algo);
    path.push(&rest[0..2]);
    path.push(&rest[2..4]);
    path.push(&rest[4..]);

    Ok(path)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_path_from_integrity() {
        let cache = "test-cache";
        let sri = Integrity::from(b"hello");
        //should be src = sha256-LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=
        let path = get_path(&cache, &sri).unwrap();
        assert_eq!(
            path.to_str().unwrap(),
            format!(
                "{}/sha256/LP/JN/ul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=",
                &cache
            )
        );
    }

    #[test]
    fn save_data_in_cache() {
        let cache = "test-cache";
        let content = b"hello";
        put(&cache, content).unwrap();
        let integrity = Integrity::from(content);
        let path = get_path(&cache, &integrity).unwrap();

        let cache_content = std::fs::read_to_string(&path).unwrap();

        assert_eq!(cache_content.as_bytes(), content);
    }
}
