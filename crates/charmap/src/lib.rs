//! # charmap
//!
//! A port of Hypothesis's internal "charmap" Python module. It provides utilities for working with
//! Unicode intervals.
//!
//! Supports Unicode 9.0 - 13.0
//!
//! ## Usage Examples:
//!
//! ```rust
//! let intervals = charmap::UnicodeVersion::V13.query(
//!     None,           // exclude categories
//!     Some(&["Lu"]),  // include categories (Uppercase letters)
//!     Some(0),        // minimum codepoint
//!     Some(128),      // maximum codepoint
//!     Some("☃"),      // include characters
//!     None            // exclude characters
//! ).expect("Invalid query input");
//! assert_eq!(intervals, &[(65, 90), (9731, 9731)])
//! ```
#![allow(clippy::redundant_static_lifetimes, clippy::unreachable)]
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
    variant_size_differences,
    clippy::integer_arithmetic,
    clippy::unwrap_used
)]
mod inner;
mod tables;
use ahash::AHashMap;
use lazy_static::lazy_static;
use std::cmp::{max, min};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Mutex;
use std::{error, fmt};

/// Unicode category abbreviation
pub type Category = &'static str;
/// Interval between two Unicode codepoints.
pub type Interval = (u32, u32);
/// Entry in the character mapping table.
pub type TableEntry = (Category, &'static [Interval]);
/// Table with character mappings.
pub type Table = [TableEntry];
/// Mapping from Unicode category abbreviations to their respective intervals.
pub type CharMap = AHashMap<Category, &'static [Interval]>;

lazy_static! {
    static ref CHARMAP_V9: CharMap = inner::make_charmap(UnicodeVersion::V9);
    static ref CHARMAP_V10: CharMap = inner::make_charmap(UnicodeVersion::V10);
    static ref CHARMAP_V11: CharMap = inner::make_charmap(UnicodeVersion::V11);
    static ref CHARMAP_V12: CharMap = inner::make_charmap(UnicodeVersion::V12);
    static ref CHARMAP_V12_1: CharMap = inner::make_charmap(UnicodeVersion::V12_1);
    static ref CHARMAP_V13: CharMap = inner::make_charmap(UnicodeVersion::V13);
    static ref CATEGORIES_V9: Vec<Category> = inner::make_categories(UnicodeVersion::V9);
    static ref CATEGORIES_V10: Vec<Category> = inner::make_categories(UnicodeVersion::V10);
    static ref CATEGORIES_V11: Vec<Category> = inner::make_categories(UnicodeVersion::V11);
    static ref CATEGORIES_V12: Vec<Category> = inner::make_categories(UnicodeVersion::V12);
    static ref CATEGORIES_V12_1: Vec<Category> = inner::make_categories(UnicodeVersion::V12_1);
    static ref CATEGORIES_V13: Vec<Category> = inner::make_categories(UnicodeVersion::V13);
}

static MAX_CODEPOINT: u32 = 1114111;

/// Supported Unicode versions
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UnicodeVersion {
    /// Unicode 9.0
    V9,
    /// Unicode 10.0
    V10,
    /// Unicode 11.0
    V11,
    /// Unicode 12.0
    V12,
    /// Unicode 12.1
    V12_1,
    /// Unicode 13.0
    V13,
}

/// Errors during Unicode intervals manipulations.
#[derive(Debug, Eq, PartialEq)]
pub enum Error<'a> {
    /// The provided category name is invalid.
    InvalidCategory(&'a str),
    /// Provided codepoints do not agree. Maximum should be greater or equal to minimum.
    InvalidCodepoints(u32, u32),
}

impl error::Error for Error<'_> {}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCategory(category) => {
                f.write_fmt(format_args!("{} is not a valid Unicode category", category))
            }
            Error::InvalidCodepoints(left, right) => f.write_fmt(format_args!(
                "Minimum codepoint should be less or equal than maximum codepoint. Got {} {}",
                left, right
            )),
        }
    }
}

impl UnicodeVersion {
    /// Get a raw table with character mappings.
    #[inline]
    pub const fn table(self) -> &'static Table {
        match self {
            UnicodeVersion::V9 => tables::v9_0_0::BY_NAME,
            UnicodeVersion::V10 => tables::v10_0_0::BY_NAME,
            UnicodeVersion::V11 => tables::v11_0_0::BY_NAME,
            UnicodeVersion::V12 => tables::v12_0_0::BY_NAME,
            UnicodeVersion::V12_1 => tables::v12_1_0::BY_NAME,
            UnicodeVersion::V13 => tables::v13_0_0::BY_NAME,
        }
    }

    /// Get a mapping from Unicode category abbreviations to their respective intervals.
    #[inline]
    pub fn charmap(self) -> &'static CharMap {
        match self {
            UnicodeVersion::V9 => CHARMAP_V9.deref(),
            UnicodeVersion::V10 => CHARMAP_V10.deref(),
            UnicodeVersion::V11 => CHARMAP_V11.deref(),
            UnicodeVersion::V12 => CHARMAP_V12.deref(),
            UnicodeVersion::V12_1 => CHARMAP_V12_1.deref(),
            UnicodeVersion::V13 => CHARMAP_V13.deref(),
        }
    }

    /// Unicode categories in a normalised order.
    #[inline]
    pub fn categories(self) -> &'static [Category] {
        let vec = match self {
            UnicodeVersion::V9 => CATEGORIES_V9.deref(),
            UnicodeVersion::V10 => CATEGORIES_V10.deref(),
            UnicodeVersion::V11 => CATEGORIES_V11.deref(),
            UnicodeVersion::V12 => CATEGORIES_V12.deref(),
            UnicodeVersion::V12_1 => CATEGORIES_V12_1.deref(),
            UnicodeVersion::V13 => CATEGORIES_V13.deref(),
        };
        vec.as_slice()
    }

    /// Expand one-letter designations of a major class to include all subclasses.
    #[inline]
    pub fn as_general_categories<'a>(
        self,
        categories: &[&'a str],
    ) -> Result<Vec<Category>, Error<'a>> {
        if !categories.is_empty() {
            let major_classes = ["L", "M", "N", "P", "S", "Z", "C"];
            let mut out = Vec::with_capacity(30);
            let mut cats: SmallVec<[&str; 30]> = categories.into();
            cats.sort_unstable();
            cats.dedup();
            let all_categories = self.categories();
            for c in &cats {
                if major_classes.contains(c) {
                    out.extend(all_categories.iter().filter(|i| i.starts_with(c)))
                } else if !all_categories.contains(c) {
                    return Err(Error::InvalidCategory(c));
                }
            }
            Ok(out)
        } else {
            Ok(vec![])
        }
    }

    /// Return a tuple of intervals covering the codepoints for all characters
    /// that meet the input criteria.
    #[inline]
    pub fn query<'a>(
        self,
        exclude_categories: Option<&[&'a str]>,
        include_categories: Option<&[&'a str]>,
        min_codepoint: Option<u32>,
        max_codepoint: Option<u32>,
        include_characters: Option<&str>,
        exclude_characters: Option<&str>,
    ) -> Result<Vec<Interval>, Error<'a>> {
        let exclude_categories = exclude_categories.unwrap_or(&[]);
        // Category validation
        let all_categories = self.categories();
        for category in exclude_categories {
            if !all_categories.contains(category) {
                return Err(Error::InvalidCategory(category));
            }
        }
        if let Some(categories) = include_categories {
            for category in categories {
                if !all_categories.contains(category) {
                    return Err(Error::InvalidCategory(category));
                }
            }
        }

        // Min codepoint <= Max codepoint
        let min_codepoint = min_codepoint.unwrap_or(0);
        let max_codepoint = max_codepoint.unwrap_or(MAX_CODEPOINT);
        if min_codepoint > max_codepoint {
            return Err(Error::InvalidCodepoints(min_codepoint, max_codepoint));
        }

        let category_key = inner::category_key(self, exclude_categories, include_categories);

        let cache_key = (
            category_key.clone(),
            min_codepoint,
            max_codepoint,
            include_characters.map(String::from),
            exclude_characters.map(String::from),
        );
        if let Ok(cache) = QUERY_CACHE.lock() {
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let include_characters = include_characters.unwrap_or("");
        let exclude_characters = exclude_characters.unwrap_or("");

        let character_intervals = inner::intervals(include_characters);
        let exclude_intervals = inner::intervals(exclude_characters);

        let base = inner::query_for_key(self, category_key.as_slice());
        let mut result = vec![];
        for (u, v) in base {
            if v >= min_codepoint && u <= max_codepoint {
                result.push((max(u, min_codepoint), min(v, max_codepoint)))
            }
        }
        let result = inner::union_intervals(result, character_intervals.as_slice());
        let result = inner::subtract_intervals(result, exclude_intervals.as_slice());
        if let Ok(mut cache) = QUERY_CACHE.lock() {
            cache.insert(cache_key, result.clone());
        }
        Ok(result)
    }
}

type QueryCacheKey = (Vec<Category>, u32, u32, Option<String>, Option<String>);

lazy_static! {
    static ref QUERY_CACHE: Mutex<AHashMap<QueryCacheKey, Vec<Interval>>> =
        Mutex::new(AHashMap::new());
}

#[cfg(feature = "benchmark")]
pub use inner::category_key;
#[cfg(feature = "benchmark")]
pub use inner::intervals;
#[cfg(feature = "benchmark")]
pub use inner::make_categories;
#[cfg(feature = "benchmark")]
pub use inner::make_charmap;
#[cfg(feature = "benchmark")]
pub use inner::query_for_key;
#[cfg(feature = "benchmark")]
pub use inner::subtract_intervals;
#[cfg(feature = "benchmark")]
pub use inner::union_intervals;
use smallvec::SmallVec;

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(&["N"], vec!["Nl", "Nd", "No"])]
    #[test_case(&["N", "N"], vec!["Nl", "Nd", "No"])]
    fn as_general_categories_work(categories: &[&'static str], expected: Vec<&str>) {
        assert_eq!(
            UnicodeVersion::V13
                .as_general_categories(categories)
                .unwrap(),
            expected
        )
    }

    #[test_case(None, None, None, None, None, None, &[(0, 1114111)])]
    #[test_case(None, None, Some(0), Some(128), None, None, &[(0, 128)])]
    #[test_case(None, Some(&["Lu"]), Some(0), Some(128), None, None, &[(65, 90)])]
    #[test_case(None, Some(&["Lu"]), Some(0), Some(128), Some("☃"), None, &[(65, 90), (9731, 9731)])]
    #[test_case(None, None, Some(0), Some(68104), Some("\u{10A07}"), None, &[(0, 68104)])]
    fn query_works(
        exclude_categories: Option<&[&str]>,
        include_categories: Option<&[&str]>,
        min_codepoint: Option<u32>,
        max_codepoint: Option<u32>,
        include_characters: Option<&str>,
        exclude_characters: Option<&str>,
        expected: &[Interval],
    ) {
        assert_eq!(
            UnicodeVersion::V13
                .query(
                    exclude_categories,
                    include_categories,
                    min_codepoint,
                    max_codepoint,
                    include_characters,
                    exclude_characters
                )
                .unwrap(),
            expected
        )
    }
}
