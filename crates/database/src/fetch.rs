use crate::Example;

#[derive(Debug)]
/// A lazily iterable result of fetching the database.
pub struct Fetch<E: IntoIterator<Item = Example>>(E);

impl<E: IntoIterator<Item = Example>> Fetch<E> {
    #[inline]
    /// Create a new `Fetch` instance.
    pub fn new(inner: E) -> Self {
        Fetch(inner)
    }

    #[inline]
    /// Create a vector of examples.
    pub fn into_vec(self) -> Vec<Example> {
        self.0.into_iter().collect()
    }
}

impl<C: IntoIterator<Item = Example>> IntoIterator for Fetch<C> {
    type Item = C::Item;
    type IntoIter = C::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
