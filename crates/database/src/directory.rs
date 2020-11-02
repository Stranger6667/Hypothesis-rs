use crate::fetch::Fetch;
use crate::{Example, ExampleDatabase, Input};
use arrayvec::ArrayString;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha384};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::fs::ReadDir;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

type Cache = HashMap<Vec<u8>, ArrayString<[u8; 16]>>;

#[derive(Debug)]
/// Use a directory to store Hypothesis examples as files.
pub struct DirectoryBasedExampleDatabase {
    /// Path to the examples database.
    pub path: PathBuf,
    cache: RefCell<Cache>,
}

macro_rules! hash {
    ($var:ident) => {{
        // Since the hash size is known statically, there is not need to allocate a `String` here
        // via using the `format!` macro. Instead we allocate on the stack
        // On average it gives ~10% performance improvement for calculating hash strings
        let mut out = ArrayString::<[_; 16]>::new();
        out.write_fmt(format_args!("{:.16x}", Sha384::digest($var)))
            .expect("Hash is always representable in hex format");
        out
    }};
}

impl DirectoryBasedExampleDatabase {
    /// Create a new example database that stores examples as files.
    pub fn new<P: AsRef<Path>>(path: P) -> DirectoryBasedExampleDatabase {
        DirectoryBasedExampleDatabase {
            path: path.as_ref().to_path_buf(),
            cache: RefCell::new(HashMap::new()),
        }
    }

    #[inline]
    fn path_for_key(&self, key: &Input) -> PathBuf {
        let mut cache = self.cache.borrow_mut();
        if let Some(hashed) = cache.get(key) {
            self.path.join(hashed.as_str())
        } else {
            let hashed = hash!(key);
            cache.insert(key.to_vec(), hashed);
            self.path.join(hashed.as_str())
        }
    }

    #[inline]
    fn path_for_value(&self, key_path: &PathBuf, value: &Input) -> PathBuf {
        let mut cache = self.cache.borrow_mut();
        if let Some(hashed) = cache.get(value) {
            key_path.join(hashed.as_str())
        } else {
            let hashed = hash!(value);
            cache.insert(value.to_vec(), hashed);
            key_path.join(hashed.as_str())
        }
    }
}

impl ExampleDatabase for DirectoryBasedExampleDatabase {
    type Source = Directory;
    #[inline]
    fn save(&mut self, key: &Input, value: &Input) {
        let key_path = self.path_for_key(key);
        if !key_path.exists() {
            fs::create_dir_all(&key_path).expect("Can't create a directory");
        }
        let value_path = self.path_for_value(&key_path, value);
        if !value_path.exists() {
            let suffix: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();
            let tmpname = value_path.with_extension(suffix);
            let mut target = fs::File::create(&tmpname).expect("Can't create a file");
            target.write_all(value).expect("Write error");
            target.sync_all().expect("Can't sync data to disk");
            if fs::rename(&tmpname, value_path).is_err() {
                fs::remove_file(&tmpname).expect("Can't remove a temporary file");
            }
        }
    }

    #[inline]
    fn delete(&mut self, key: &Input, value: &Input) {
        let key_path = self.path_for_key(key);
        let value_path = self.path_for_value(&key_path, value);
        let _ = fs::remove_file(&value_path);
    }

    #[inline]
    fn r#move(&mut self, src: &Input, dst: &Input, value: &Input) {
        if src == dst {
            self.save(src, value)
        } else {
            let src_path = self.path_for_key(src);
            let dst_path = self.path_for_key(dst);
            if fs::rename(
                self.path_for_value(&src_path, value),
                self.path_for_value(&dst_path, value),
            )
            .is_err()
            {
                self.delete(src, value);
                self.save(dst, value)
            }
        }
    }

    #[inline]
    fn fetch(&self, key: &Input) -> Fetch<Self::Source> {
        Fetch::new(Directory {
            path: self.path_for_key(key),
        })
    }
}

#[derive(Debug)]
pub struct Directory {
    path: PathBuf,
}

#[derive(Debug)]
/// Iterates over files in a directory and yields examples.
pub struct FileIterator {
    entries: Option<ReadDir>,
}

impl Iterator for FileIterator {
    type Item = Example;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entries) = &mut self.entries {
            if let Some(Ok(entry)) = entries.next() {
                if let Ok(mut file) = fs::File::open(entry.path()) {
                    // We don't need buffered reads here, we'd like to make one read
                    let mut contents = Vec::with_capacity(32);
                    if file.read_to_end(&mut contents).is_ok() {
                        return Some(contents);
                    }
                }
            }
            None
        } else {
            None
        }
    }
}

impl IntoIterator for Directory {
    type Item = Example;
    type IntoIter = FileIterator;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let entries = if !self.path.exists() {
            None
        } else {
            Some(fs::read_dir(self.path).expect("Can't read the directory"))
        };
        FileIterator { entries }
    }
}

#[cfg(feature = "benchmark")]
/// Benchmark-only function to measure the `hash!` performance
pub fn calculate_hash(value: &Input) {
    hash!(value);
}

#[cfg(test)]
pub(crate) mod tests_util {
    use super::*;
    use tempdir::TempDir;

    pub(crate) struct TestDatabase {
        _temp: TempDir,
        db: DirectoryBasedExampleDatabase,
        pub(crate) path: String,
    }

    impl TestDatabase {
        pub(crate) fn new() -> Self {
            let dir = TempDir::new("test-db").expect("Should always work");
            let path = dir.path().to_str().expect("This path is UTF-8").to_string();
            let db = DirectoryBasedExampleDatabase::new(path.clone());
            TestDatabase {
                _temp: dir,
                db,
                path,
            }
        }

        pub(crate) fn from_string(path: String) -> Self {
            let db = DirectoryBasedExampleDatabase::new(path.clone());
            TestDatabase {
                _temp: TempDir::new("test-db").expect("Should always work"),
                db,
                path,
            }
        }
    }

    impl ExampleDatabase for TestDatabase {
        type Source = Directory;

        fn save(&mut self, key: &Input, value: &Input) {
            self.db.save(key, value)
        }

        fn delete(&mut self, key: &Input, value: &Input) {
            self.db.delete(key, value)
        }

        fn r#move(&mut self, src: &Input, dst: &Input, value: &Input) {
            self.db.r#move(src, dst, value)
        }

        fn fetch(&self, key: &Input) -> Fetch<Self::Source> {
            self.db.fetch(key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_directory_databases_can_interact() {
        let mut db1 = tests_util::TestDatabase::new();
        let mut db2 = tests_util::TestDatabase::from_string(db1.path.clone());
        assert_eq!(db1.path, db2.path);
        db1.save(b"foo", b"bar");
        assert_eq!(db2.fetch(b"foo").into_vec(), vec![b"bar"]);
        db2.save(b"foo", b"bar");
        db2.save(b"foo", b"baz");
        let mut result = db1.fetch(b"foo").into_vec();
        result.sort_unstable();
        assert_eq!(result, vec![b"bar", b"baz"]);
    }
}
