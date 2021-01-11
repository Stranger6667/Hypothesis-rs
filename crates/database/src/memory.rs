use crate::fetch::Fetch;
use crate::{Example, ExampleDatabase, Input};
use ahash::{AHashMap, AHashSet};

#[derive(Debug)]
/// A non-persistent example database, implemented in terms of a hashmaps of sets.
pub struct InMemoryExampleDatabase {
    data: AHashMap<Vec<u8>, AHashSet<Example>>,
}

impl InMemoryExampleDatabase {
    /// Create a new non-persistent example database.
    pub fn new() -> InMemoryExampleDatabase {
        InMemoryExampleDatabase {
            data: AHashMap::with_capacity(64),
        }
    }
}
impl Default for InMemoryExampleDatabase {
    fn default() -> InMemoryExampleDatabase {
        InMemoryExampleDatabase::new()
    }
}

impl ExampleDatabase for InMemoryExampleDatabase {
    type Source = Vec<Example>;
    #[inline]
    fn save(&mut self, key: &Input, value: &Input) {
        self.data
            .entry(key.to_vec())
            .or_insert_with(AHashSet::new)
            .insert(value.to_vec());
    }

    #[inline]
    fn delete(&mut self, key: &Input, value: &Input) {
        if let Some(entry) = self.data.get_mut(key) {
            entry.remove(value);
        }
    }

    #[inline]
    fn fetch(&self, key: &Input) -> Fetch<Self::Source> {
        // Collecting into a vector is faster than cloning a hashset
        Fetch::new(
            self.data
                .get(key)
                .map_or_else(Vec::new, |hs| hs.iter().cloned().collect()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_delete_keys() {
        let mut db = InMemoryExampleDatabase::new();
        db.save(b"foo", b"bar");
        db.save(b"foo", b"baz");
        db.delete(b"foo", b"bar");
        assert_eq!(db.fetch(b"foo").into_vec(), vec![b"baz".to_vec()])
    }

    #[test]
    fn test_does_not_error_when_fetching_when_not_exist() {
        let db = InMemoryExampleDatabase::new();
        let expected: Vec<Vec<u8>> = vec![];
        assert_eq!(db.fetch(b"foo").into_vec(), expected);
    }
}
