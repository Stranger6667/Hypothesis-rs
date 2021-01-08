use crate::{Category, CharMap, Interval, TableEntry, UnicodeVersion, MAX_CODEPOINT};
use ahash::{AHashMap, AHashSet};
use lazy_static::lazy_static;
use smallvec::SmallVec;
use std::cmp::max;
use std::convert::TryInto;
use std::sync::Mutex;

#[inline]
pub fn make_charmap(version: UnicodeVersion) -> CharMap {
    let mut map = CharMap::with_capacity(30);
    for (name, data) in version.table() {
        map.insert(name, data);
    }
    map
}

#[inline]
pub fn make_categories(version: UnicodeVersion) -> Vec<Category> {
    // Note. all this can be done at compile time
    let mut table: [TableEntry; 30] = version
        .table()
        .try_into()
        .expect("Table category size should be 30");
    table.sort_unstable_by_key(|(_, v)| v.len());
    let mut out: Vec<&str> = table
        .iter()
        .map(|(cat, _)| *cat)
        .filter(|cat| !["Cc", "Cs"].contains(cat))
        .collect();
    out.extend_from_slice(&["Cc", "Cs"]);
    out
}

#[inline]
// Practically all interval values are < u32::MAX
// Therefore there will be no panic (debug) / wrapping (release)
#[allow(clippy::integer_arithmetic)]
pub fn union_intervals(x: &[Interval], y: &[Interval]) -> Vec<Interval> {
    if x.is_empty() {
        y.to_vec()
    } else if y.is_empty() {
        x.to_vec()
    } else {
        let mut intervals = [x, y].concat();
        // Separate `sort` and `reverse` calls are generally faster than `sort_by_key` with `Reverse`
        // Note! merge sort is faster than quicksort on the test dataset - worth exploring why
        #[allow(clippy::stable_sort_primitive)]
        intervals.sort();
        intervals.reverse();
        let mut result = Vec::with_capacity(16);
        result.push(intervals.pop().expect("It is not empty"));
        while let Some((u, v)) = intervals.pop() {
            let (_, b) = result.last_mut().expect("It is not empty");
            if u <= *b + 1 {
                *b = max(v, *b);
            } else {
                result.push((u, v))
            }
        }
        result
    }
}

#[inline]
// Practically all interval values are < u32::MAX
// Therefore there will be no panic (debug) / wrapping (release)
#[allow(clippy::integer_arithmetic)]
pub fn subtract_intervals(mut x: Vec<Interval>, y: &[Interval]) -> Vec<Interval> {
    if y.is_empty() {
        x
    } else {
        let (mut i, mut j) = (0usize, 0usize);
        let mut result = Vec::with_capacity(x.len());
        while i < x.len() && j < y.len() {
            let (xl, xr) = x[i];
            let (yl, yr) = y[j];
            if yr < xl {
                j += 1;
            } else if yl > xr {
                result.push(x[i]);
                i += 1;
            } else if yl <= xl {
                if yr >= xr {
                    i += 1;
                } else {
                    x[i].0 = yr + 1;
                    j += 1
                }
            } else {
                result.push((xl, yl - 1));
                if yr < xr {
                    x[i].0 = yr + 1;
                    j += 1;
                } else {
                    i += 1;
                }
            }
        }
        result.extend_from_slice(&x[i..]);
        result
    }
}

#[inline]
#[allow(clippy::integer_arithmetic)]
pub fn intervals(string: &str) -> Vec<Interval> {
    if string.is_empty() {
        return vec![];
    }
    let mut intervals: SmallVec<[Interval; 32]> =
        string.chars().map(|c| (c as u32, c as u32)).collect();
    #[allow(clippy::stable_sort_primitive)]
    intervals.sort();
    intervals.reverse();
    let mut result = Vec::with_capacity(16);
    result.push(intervals.pop().expect("It is not empty"));
    while let Some((u, v)) = intervals.pop() {
        let (_, b) = result.last_mut().expect("It is not empty");
        if u <= *b + 1 {
            *b = max(v, *b);
        } else {
            result.push((u, v))
        }
    }
    result
}

/// Return a normalised tuple of all Unicode categories that are in `include`, but not in `exclude`.
#[inline]
pub fn category_key(
    version: UnicodeVersion,
    exclude: &[&str],
    include: Option<&[&str]>,
) -> Vec<Category> {
    let cs = version.categories();
    // The default `collect` will dynamically grow the vector in multiple allocations, but we know
    // the maximum capacity upfront and can avoid a few allocations
    let mut out = Vec::with_capacity(30);
    if let Some(include) = include {
        if exclude.is_empty() {
            out.extend(cs.iter().filter(|c| include.contains(c)).copied())
        } else {
            let include: SmallVec<[&&str; 30]> =
                include.iter().filter(|c| !exclude.contains(c)).collect();
            out.extend(cs.iter().filter(|c| include.contains(c)).copied());
        }
    } else if exclude.is_empty() {
        out.extend_from_slice(cs)
    } else {
        out.extend(cs.iter().filter(|c| !exclude.contains(c)).copied())
    };
    out
}

lazy_static! {
    static ref CATEGORY_INDEX_CACHE: Mutex<AHashMap<Vec<Category>, Vec<Interval>>> =
        Mutex::new(AHashMap::new());
}

#[inline]
pub fn query_for_key(version: UnicodeVersion, key: &[Category]) -> Vec<Interval> {
    // NOTE. The right hand side can be calculated via lazy_static
    if key.iter().collect::<AHashSet<&Category>>()
        == version.categories().iter().collect::<AHashSet<&Category>>()
    {
        return vec![(0, MAX_CODEPOINT)];
    }
    if let Some((last, left)) = key.split_last() {
        if let Ok(cache) = CATEGORY_INDEX_CACHE.lock() {
            if let Some(cached) = cache.get(key) {
                return cached.clone();
            }
        }
        let left = query_for_key(version, left);
        // `last` is a valid Unicode category name; therefore it always exists in `charmap`
        let right = version
            .charmap()
            .get(last)
            .expect("It should be a valid Unicode category");
        let result = union_intervals(left.as_slice(), right);
        if let Ok(mut cache) = CATEGORY_INDEX_CACHE.lock() {
            cache.insert(key.to_vec(), result.clone());
        }
        result
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_intervals_empty() {
        assert_eq!(union_intervals(&[], &[]), &[]);
        assert_eq!(union_intervals(&[], &[(1, 2)]), &[(1, 2)]);
        assert_eq!(union_intervals(&[(1, 2)], &[]), &[(1, 2)]);
    }

    #[test]
    fn union_handles_totally_overlapped_gap() {
        assert_eq!(union_intervals(&[(2, 3)], &[(1, 2), (4, 5)]), &[(1, 5)])
    }

    #[test]
    fn union_handles_partially_overlapped_gap() {
        assert_eq!(
            union_intervals(&[(3, 3)], &[(1, 2), (5, 5)]),
            &[(1, 3), (5, 5)]
        )
    }

    #[test]
    fn subtract_works() {
        assert_eq!(subtract_intervals(vec![(0, 1), (3, 3)], &[(0, 3)]), &[]);
        assert_eq!(
            subtract_intervals(vec![(0, 1), (3, 3)], &[(1, 3)]),
            &[(0, 0)]
        );
        assert_eq!(
            subtract_intervals(vec![(0, 10)], &[(2, 3), (9, 15)]),
            &[(0, 1), (4, 8)]
        );
    }

    #[test]
    fn intervals_works() {
        assert_eq!(intervals("aa"), &[(97, 97)]);
        assert_eq!(intervals("abcdef0123456789"), &[(48, 57), (97, 102)]);
        assert_eq!(intervals("01234fedcba98765"), &[(48, 57), (97, 102)]);
    }

    #[test]
    fn test_category_key() {
        assert_eq!(
            category_key(
                UnicodeVersion::V13,
                &["So"],
                Some(&["Lu", "Me", "Cs", "So"])
            ),
            vec!["Me", "Lu", "Cs"]
        );
    }

    #[test]
    fn test_query_for_key() {
        assert_eq!(
            query_for_key(UnicodeVersion::V13, &["Zl", "Zp", "Co"]),
            vec![
                (8232, 8233),
                (57344, 63743),
                (983040, 1048573),
                (1048576, 1114109)
            ]
        );
    }

    #[test]
    fn test_query_for_key_all() {
        assert_eq!(
            query_for_key(
                UnicodeVersion::V13,
                &[
                    "Pe", "Pc", "Cc", "Sc", "Pd", "Nd", "Me", "Pf", "Cf", "Pi", "Nl", "Zl", "Ll",
                    "Sm", "Lm", "Sk", "Mn", "Ps", "Lo", "No", "Po", "So", "Zp", "Co", "Zs", "Mc",
                    "Cs", "Lt", "Cn", "Lu"
                ]
            ),
            vec![(0, MAX_CODEPOINT),]
        );
        // Non-default order
        assert_eq!(
            query_for_key(
                UnicodeVersion::V13,
                &[
                    "Sm", "Lm", "Sk", "Mn", "Ps", "Lo", "No", "Po", "So", "Zp", "Co", "Zs", "Mc",
                    "Pe", "Pc", "Cc", "Sc", "Pd", "Nd", "Me", "Pf", "Cf", "Pi", "Nl", "Zl", "Ll",
                    "Cs", "Lt", "Cn", "Lu"
                ]
            ),
            vec![(0, MAX_CODEPOINT),]
        );
        // Duplicated categories
        assert_eq!(
            query_for_key(
                UnicodeVersion::V13,
                &[
                    "Sm", "Lm", "Sk", "Mn", "Ps", "Lo", "No", "Po", "So", "Zp", "Co", "Zs", "Mc",
                    "Pe", "Pc", "Cc", "Sc", "Pd", "Nd", "Me", "Pf", "Cf", "Pi", "Nl", "Zl", "Ll",
                    "Cs", "Lt", "Cn", "Lu", "Lu", "Sm"
                ]
            ),
            vec![(0, MAX_CODEPOINT),]
        );
    }

    #[test]
    fn successive_union() {
        let mut x = vec![];
        for v in UnicodeVersion::V13.charmap().values() {
            x = union_intervals(x.as_slice(), v);
        }
        assert_eq!(x, vec![(0, MAX_CODEPOINT)])
    }
}
