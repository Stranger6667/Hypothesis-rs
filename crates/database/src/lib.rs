//! Proof-of-concept implementation of example databases for Hypothesis.
#![warn(
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::needless_pass_by_value,
    clippy::print_stdout,
    clippy::redundant_closure,
    clippy::trivially_copy_pass_by_ref,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unreachable_pub,
    variant_size_differences,
    clippy::integer_arithmetic,
    clippy::unwrap_used
)]

mod directory;
mod fetch;
mod memory;
pub use directory::{DirectoryBasedExampleDatabase, FileIterator};
pub use fetch::Fetch;
pub use memory::InMemoryExampleDatabase;

/// Any input that the database can work with.
pub type Input = [u8];
/// Example that is returned by the database.
pub type Example = Vec<u8>;

/// Trait for storing examples in Hypothesis' internal format.
pub trait ExampleDatabase {
    /// The type of example source. It could be anything, that can iterate over examples
    type Source: IntoIterator<Item = Example>;

    /// Save `value` under `key`.
    /// If this value is already present for this key, silently do nothing
    fn save(&mut self, key: &Input, value: &Input);

    /// Remove this `value` from this `key`.
    /// If this value is not present, silently do nothing.
    fn delete(&mut self, key: &Input, value: &Input);

    /// Move `value` from key `src` to key `dst`.
    #[inline]
    fn r#move(&mut self, src: &Input, dst: &Input, value: &Input) {
        if src == dst {
            self.save(src, value)
        } else {
            self.delete(src, value);
            self.save(dst, value)
        }
    }

    /// Return an iterable over all values matching this key.
    fn fetch(&self, key: &Input) -> Fetch<Self::Source>;
}

#[cfg(feature = "benchmark")]
pub use directory::calculate_hash;

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(InMemoryExampleDatabase::new())]
    #[test_case(directory::tests_util::TestDatabase::new())]
    fn can_iterate(mut db: impl ExampleDatabase) {
        db.save(b"foo", b"bar");
        for example in db.fetch(b"foo") {
            assert_eq!(example, b"bar")
        }
        assert_eq!(db.fetch(b"foo").into_vec(), vec![b"bar"])
    }

    #[test_case(InMemoryExampleDatabase::new())]
    #[test_case(directory::tests_util::TestDatabase::new())]
    fn can_delete_a_key_that_is_not_present(mut db: impl ExampleDatabase) {
        db.delete(b"foo", b"bar");
    }

    #[test_case(&InMemoryExampleDatabase::new())]
    #[test_case(&directory::tests_util::TestDatabase::new())]
    fn can_fetch_a_key_that_is_not_present(db: &impl ExampleDatabase) {
        assert_eq!(db.fetch(b"bar").into_vec().len(), 0)
    }

    #[test_case(InMemoryExampleDatabase::new())]
    #[test_case(directory::tests_util::TestDatabase::new())]
    fn saving_a_key_twice_fetches_it_once(mut db: impl ExampleDatabase) {
        db.save(b"foo", b"bar");
        db.save(b"foo", b"bar");
        assert_eq!(db.fetch(b"foo").into_vec(), vec![b"bar"])
    }

    #[test_case(InMemoryExampleDatabase::new())]
    #[test_case(directory::tests_util::TestDatabase::new())]
    fn an_absent_value_is_present_after_it_moves(mut db: impl ExampleDatabase) {
        db.r#move(b"a", b"b", b"c");
        assert_eq!(db.fetch(b"b").into_vec(), vec![b"c"])
    }

    #[test_case(InMemoryExampleDatabase::new())]
    #[test_case(directory::tests_util::TestDatabase::new())]
    fn an_absent_value_is_present_after_it_moves_to_self(mut db: impl ExampleDatabase) {
        db.r#move(b"a", b"a", b"b");
        assert_eq!(db.fetch(b"a").into_vec(), vec![b"b"])
    }

    #[test_case(InMemoryExampleDatabase::new())]
    #[test_case(directory::tests_util::TestDatabase::new())]
    fn appears_in_listing_after_saving(mut db: impl ExampleDatabase) {
        db.save(b"foo", b"bar");
        assert_eq!(db.fetch(b"foo").into_vec(), vec![b"bar"])
    }

    #[test_case(InMemoryExampleDatabase::new())]
    #[test_case(directory::tests_util::TestDatabase::new())]
    fn can_delete_key(mut db: impl ExampleDatabase) {
        db.save(b"foo", b"bar");
        db.delete(b"foo", b"bar");
        assert_eq!(db.fetch(b"foo").into_vec().len(), 0);
    }
}
