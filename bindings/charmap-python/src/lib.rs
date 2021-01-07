use charmap::{CategoryBitMap, Error};
use lazy_static::lazy_static;
use pyo3::exceptions::{PyAssertionError, PyRuntimeError, PyTypeError};
use pyo3::types::{PyBytes, PyDict, PyInt, PyString, PyTuple};
use pyo3::AsPyPointer;
use pyo3::{ffi, prelude::*, wrap_pyfunction};
use std::sync::Mutex;

#[allow(dead_code)]
mod build {
    // Include the content of the `built.rs` file here
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

lazy_static! {
    static ref UNICODE_VERSION: Mutex<Option<charmap::UnicodeVersion>> = Mutex::new(None);
    static ref CHARMAP_CACHE: Mutex<Option<PyObject>> = Mutex::new(None);
    static ref CATEGORIES_CACHE: Mutex<Option<PyObject>> = Mutex::new(None);
}

#[inline]
fn get_unicode_version() -> charmap::UnicodeVersion {
    if let Ok(cached) = UNICODE_VERSION.lock() {
        return cached.expect("Set during import");
    };
    panic!("Lock is poisoned!")
}

/// A port of Hypothesis's internal "charmap" Python module. It provides utilities for working with
/// Unicode intervals. Supports Unicode 9.0 - 13.0
#[pymodule]
fn charmap(py: Python, module: &PyModule) -> PyResult<()> {
    let unicodedata = py.import("unicodedata")?;
    let unicode_version: String = unicodedata.get("unidata_version")?.extract()?;
    let version = match unicode_version.as_str() {
        "9.0.0" => charmap::UnicodeVersion::V9,
        "10.0.0" => charmap::UnicodeVersion::V10,
        "11.0.0" => charmap::UnicodeVersion::V11,
        "12.0.0" => charmap::UnicodeVersion::V12,
        "12.1.0" => charmap::UnicodeVersion::V12_1,
        "13.0.0" => charmap::UnicodeVersion::V13,
        _ => return Err(PyRuntimeError::new_err("Unsupported unicode version")),
    };

    if let Ok(mut cached) = UNICODE_VERSION.lock() {
        *cached = Some(version);
    };

    module.add("__build__", pyo3_built::pyo3_built!(py, build))?;

    /// charmap()
    ///
    /// >>> charmap()['Co']
    /// ((57344, 63743), (983040, 1048573), (1048576, 1114109))
    ///
    /// Returns
    /// -------
    /// charmap : dict[str, tuple]
    ///
    /// Get a mapping from Unicode category abbreviations to their respective intervals.
    #[pyfunction]
    #[text_signature = "()"]
    fn charmap(py: Python) -> PyResult<&PyDict> {
        if let Ok(cached) = CHARMAP_CACHE.lock() {
            if let Some(obj) = cached.as_ref() {
                return Ok(unsafe { py.from_borrowed_ptr::<PyDict>(obj.as_ptr()) });
            }
        };
        let result = unsafe { py.from_owned_ptr::<PyDict>(ffi::_PyDict_NewPresized(30isize)) };
        for (name, value) in get_unicode_version().table() {
            result
                .set_item(name, PyTuple::new(py, *value))
                .expect("Can't create a charmap");
        }
        if let Ok(mut cached) = CHARMAP_CACHE.lock() {
            *cached = Some(result.into());
        };
        Ok(result)
    }

    module.add_wrapped(wrap_pyfunction!(charmap))?;

    /// categories()
    ///
    /// >>> categories()
    /// ("Zp", "Zl", "Co", ...)
    ///
    /// Returns
    /// -------
    /// categories : tuple
    ///
    /// Unicode categories in a normalised order.
    #[pyfunction]
    #[text_signature = "()"]
    fn categories(py: Python) -> PyResult<&PyTuple> {
        if let Ok(cached) = CATEGORIES_CACHE.lock() {
            if let Some(obj) = cached.as_ref() {
                return Ok(unsafe { py.from_borrowed_ptr::<PyTuple>(obj.as_ptr()) });
            }
        };
        let categories = get_unicode_version().categories();
        let result = PyTuple::new(py, categories);
        if let Ok(mut cached) = CATEGORIES_CACHE.lock() {
            *cached = Some(result.into());
        };
        Ok(result)
    }
    module.add_wrapped(wrap_pyfunction!(categories))?;

    /// as_general_categories(categories, name="cats")
    ///
    /// Returns
    /// -------
    /// as_general_categories : tuple
    ///
    /// Expand one-letter designations of a major class to include all subclasses.
    #[pyfunction]
    #[text_signature = "(categories, name=\"cats\")"]
    fn as_general_categories<'p>(
        py: Python<'p>,
        categories: Vec<&PyString>,
        name: Option<&str>,
    ) -> PyResult<&'p PyTuple> {
        let mut x = CategoryBitMap::new();
        for cat in categories.iter() {
            if let Ok(s) = cat.to_str() {
                x.try_extend(s).map_err(|err| {
                    let name = name.unwrap_or("cats");
                    match err {
                        Error::InvalidCategory(category) => PyTypeError::new_err(format!(
                            "In {}={:?}, '{}' is not a valid Unicode category.",
                            name, categories, category
                        )),
                        Error::InvalidCodepoints(left, right) => {
                            PyAssertionError::new_err(format!("{} < {}", left, right))
                        }
                    }
                })?
            } else {
                let bytes = unsafe {
                    py.from_owned_ptr::<PyBytes>(ffi::PyUnicode_AsEncodedString(
                        cat.as_ptr(),
                        b"unicode-escape\0" as *const _ as _,
                        b"surrogatepass\0" as *const _ as _,
                    ))
                };
                let category = String::from_utf8_lossy(bytes.as_bytes());
                let name = name.unwrap_or("cats");
                return Err(PyTypeError::new_err(format!(
                    "In {}={:?}, '{}' is not a valid Unicode category.",
                    name, categories, category
                )));
            }
        }

        Ok(PyTuple::new(py, x.iter()))
            // get_unicode_version()
            //     .as_general_categories(out.as_slice())
            .map_err(|err| {
                let name = name.unwrap_or("cats");
                match err {
                    Error::InvalidCategory(category) => PyTypeError::new_err(format!(
                        "In {}={:?}, '{}' is not a valid Unicode category.",
                        name, categories, category
                    )),
                    Error::InvalidCodepoints(left, right) => {
                        PyAssertionError::new_err(format!("{} < {}", left, right))
                    }
                }
            })
        //     .map(|cats| PyTuple::new(py, cats))
    }
    module.add_wrapped(wrap_pyfunction!(as_general_categories))?;

    /// query(exclude_categories=(), include_categories=None, min_codepoint=None, max_codepoint=None, include_characters='', exclude_characters='')
    ///
    /// Returns
    /// -------
    /// query : tuple
    ///
    /// Return a tuple of intervals covering the codepoints for all characters
    /// that meet the input criteria.
    #[pyfunction]
    #[text_signature = "(exclude_categories=(), include_categories=None, min_codepoint=None, max_codepoint=None, include_characters='', exclude_characters='')"]
    fn query<'p>(
        py: Python<'p>,
        exclude_categories: Option<&PyAny>,
        include_categories: Option<&PyAny>,
        min_codepoint: Option<&PyInt>,
        max_codepoint: Option<&PyInt>,
        include_characters: Option<&str>,
        exclude_characters: Option<&str>,
    ) -> PyResult<&'p PyTuple> {
        // The following conversion is here to match the original Python implementation behavior
        let exclude_categories = if let Some(cats) = exclude_categories {
            if let Ok(iter) = cats.iter() {
                let mut out = CategoryBitMap::new();
                for item in iter {
                    if let Ok(Ok(x)) = item.map(|i| i.extract::<&str>()) {
                        out.try_extend(x).unwrap()
                    } else {
                        return Err(PyTypeError::new_err(
                            "Expected an iterable of valid Unicode categories",
                        ));
                    }
                }
                Some(out)
            } else {
                None
            }
        } else {
            None
        };
        let include_categories = if let Some(cats) = include_categories {
            if let Ok(iter) = cats.iter() {
                let mut out = CategoryBitMap::new();
                for item in iter {
                    if let Ok(Ok(x)) = item.map(|i| i.extract::<&str>()) {
                        out.try_extend(x).unwrap()
                    } else {
                        return Err(PyTypeError::new_err(
                            "Expected an iterable of valid Unicode categories",
                        ));
                    }
                }
                Some(out)
            } else {
                None
            }
        } else {
            None
        };
        let min_codepoint = min_codepoint.map(|x| x.extract::<u32>().unwrap_or(0));
        let max_codepoint = max_codepoint.map(|x| x.extract::<u32>().unwrap_or(0));
        let result = get_unicode_version().query(
            exclude_categories,
            include_categories,
            min_codepoint,
            max_codepoint,
            include_characters,
            exclude_characters,
        );
        match result {
            Ok(result) => Ok(PyTuple::new(py, result)),
            Err(e) => match e {
                Error::InvalidCategory(category) => Err(PyTypeError::new_err(format!(
                    "'{}' is not a valid Unicode category.",
                    category
                ))),
                Error::InvalidCodepoints(left, right) => {
                    // Simulates behavior of the Python version of `charmap`
                    Err(PyAssertionError::new_err(format!("{} < {}", left, right)))
                }
            },
        }
    }
    module.add_wrapped(wrap_pyfunction!(query))?;

    Ok(())
}
