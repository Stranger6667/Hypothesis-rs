use crate::{Category, CharMap, Interval, TableEntry, UnicodeVersion, MAX_CODEPOINT};
use ahash::{AHashMap, AHashSet};
use lazy_static::lazy_static;
use smallvec::SmallVec;
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
pub fn union_intervals(mut left: Vec<Interval>, right: Vec<Interval>) -> Vec<Interval> {
    if left.is_empty() {
        right
    } else if right.is_empty() {
        left
    } else {
        left.extend(right);
        merge_intervals(&mut left).to_vec()
    }
}

#[inline]
// Practically all interval values are < u32::MAX
// Therefore there will be no panic (debug) / wrapping (release)
#[allow(clippy::integer_arithmetic)]
pub fn subtract_intervals(mut left: Vec<Interval>, right: &[Interval]) -> Vec<Interval> {
    if right.is_empty() {
        left
    } else {
        let (mut i, mut j) = (0usize, 0usize);
        let mut result = Vec::with_capacity(left.len());
        while i < left.len() && j < right.len() {
            let (ll, lr) = left[i];
            let (rl, rr) = right[j];
            if rr < ll {
                j += 1;
            } else if rl > lr {
                result.push(left[i]);
                i += 1;
            } else if rl <= ll {
                if rr >= lr {
                    i += 1;
                } else {
                    left[i].0 = rr + 1;
                    j += 1
                }
            } else {
                result.push((ll, rl - 1));
                if rr < lr {
                    left[i].0 = rr + 1;
                    j += 1;
                } else {
                    i += 1;
                }
            }
        }
        result.extend_from_slice(&left[i..]);
        result
    }
}

#[inline]
pub fn intervals(string: &str) -> Vec<Interval> {
    if string.is_empty() {
        return vec![];
    }
    let mut intervals: SmallVec<[Interval; 32]> =
        string.chars().map(|c| (c as u32, c as u32)).collect();
    merge_intervals(&mut intervals).to_vec()
}

// Note, `#[inline]` leads to worse performance
// Practically all interval values are < u32::MAX
// Therefore there will be no panic (debug) / wrapping (release)
#[allow(clippy::integer_arithmetic)]
fn merge_intervals(intervals: &mut [Interval]) -> &[Interval] {
    // Note! merge sort is faster than quicksort on the test dataset - worth exploring why
    #[allow(clippy::stable_sort_primitive)]
    intervals.sort_by_key(|a| a.0);
    let mut border = 0usize;
    for index in 1..intervals.len() {
        let interval = intervals[index];
        if interval.0 <= intervals[border].1 + 1 {
            // Intervals overlap
            if interval.1 > intervals[border].1 {
                // Extend the one behind the border only if the current candidate right border
                // is greater
                intervals[border].1 = interval.1;
            }
        } else {
            // No overlap, this interval should be next behind the border
            border += 1;
            intervals[border] = interval;
        }
    }
    &intervals[..=border]
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
            .expect("It should be a valid Unicode category")
            .to_vec();
        let result = union_intervals(left, right);
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
        assert_eq!(union_intervals(vec![], vec![]), &[]);
        assert_eq!(union_intervals(vec![], vec![(1, 2)]), &[(1, 2)]);
        assert_eq!(union_intervals(vec![(1, 2)], vec![]), &[(1, 2)]);
    }

    #[test]
    fn union_handles_totally_overlapped_gap() {
        assert_eq!(
            union_intervals(vec![(2, 3)], vec![(1, 2), (4, 5)]),
            &[(1, 5)]
        )
    }

    #[test]
    fn union_handles_partially_overlapped_gap() {
        assert_eq!(
            union_intervals(vec![(3, 3)], vec![(1, 2), (5, 5)]),
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
        assert_eq!(intervals("\u{10A07}"), &[(68103, 68103)]);
        assert_eq!(intervals("a"), &[(97, 97)]);
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
            x = union_intervals(x, v.to_vec());
        }
        assert_eq!(x, vec![(0, MAX_CODEPOINT)])
    }
}
