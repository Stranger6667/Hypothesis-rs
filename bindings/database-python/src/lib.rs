//! Python bindings for `database`.
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
use database as db;
use database::ExampleDatabase;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::PyIterProtocol;

#[pyclass(module = "database")]
struct InMemoryExampleDatabase {
    inner: db::InMemoryExampleDatabase,
}

#[pymethods]
impl InMemoryExampleDatabase {
    #[new]
    /// Create a new non-persistent example database.
    fn new() -> Self {
        Self {
            inner: db::InMemoryExampleDatabase::new(),
        }
    }

    /// Save `value` under `key`.
    /// If this value is already present for this key, silently do nothing
    #[inline]
    fn save(&mut self, key: &[u8], value: &[u8]) -> PyResult<()> {
        self.inner.save(key, value);
        Ok(())
    }
    /// Remove this `value` from this `key`.
    /// If this value is not present, silently do nothing.
    #[inline]
    fn delete(&mut self, key: &[u8], value: &[u8]) -> PyResult<()> {
        self.inner.delete(key, value);
        Ok(())
    }

    /// Move `value` from key `src` to key `dst`.
    #[inline]
    fn r#move(&mut self, src: &[u8], dst: &[u8], value: &[u8]) -> PyResult<()> {
        self.inner.r#move(src, dst, value);
        Ok(())
    }

    /// Return an iterable over all values matching this key.
    #[inline]
    fn fetch(&self, key: &[u8]) -> PyResult<InMemoryFetch> {
        Ok(InMemoryFetch {
            inner: self.inner.fetch(key).into_iter(),
        })
    }
}

#[pyclass(module = "database")]
struct InMemoryFetch {
    inner: std::vec::IntoIter<db::Example>,
}

#[pyproto]
impl PyIterProtocol for InMemoryFetch {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<&PyBytes> {
        let py = unsafe { Python::assume_gil_acquired() };
        slf.inner.next().as_ref().map(|e| PyBytes::new(py, e))
    }
}

#[pyclass(module = "database")]
struct DirectoryBasedExampleDatabase {
    inner: db::DirectoryBasedExampleDatabase,
}

#[pymethods]
impl DirectoryBasedExampleDatabase {
    #[new]
    /// Create a new example database that stores examples as files.
    fn new(path: String) -> Self {
        Self {
            inner: db::DirectoryBasedExampleDatabase::new(path),
        }
    }

    /// Save `value` under `key`.
    /// If this value is already present for this key, silently do nothing
    fn save(&mut self, key: &[u8], value: &[u8]) -> PyResult<()> {
        self.inner.save(key, value);
        Ok(())
    }
    /// Remove this `value` from this `key`.
    /// If this value is not present, silently do nothing.
    fn delete(&mut self, key: &[u8], value: &[u8]) -> PyResult<()> {
        self.inner.delete(key, value);
        Ok(())
    }

    /// Move `value` from key `src` to key `dst`.
    fn r#move(&mut self, src: &[u8], dst: &[u8], value: &[u8]) -> PyResult<()> {
        self.inner.r#move(src, dst, value);
        Ok(())
    }

    /// Return an iterable over all values matching this key.
    fn fetch(&self, key: &[u8]) -> PyResult<DirectoryFetch> {
        Ok(DirectoryFetch {
            inner: self.inner.fetch(key).into_iter(),
        })
    }
}

#[pyclass(module = "database")]
struct DirectoryFetch {
    inner: db::FileIterator,
}

#[pyproto]
impl PyIterProtocol for DirectoryFetch {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<&PyBytes> {
        let py = unsafe { Python::assume_gil_acquired() };
        slf.inner.next().as_ref().map(|e| PyBytes::new(py, e))
    }
}

#[pymodule]
fn database(_: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<InMemoryExampleDatabase>()?;
    module.add_class::<DirectoryBasedExampleDatabase>()?;
    Ok(())
}
