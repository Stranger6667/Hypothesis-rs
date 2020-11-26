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
pub type Interval = (u32, u32);

// TODO. add benchmarks
// TODO. add Python bindings

#[derive(Debug)]
pub struct IntervalSet {
    intervals: Vec<Interval>,
    offsets: Vec<u32>,
    pub size: u32,
}

impl IntervalSet {
    pub fn new(intervals: &[Interval]) -> Self {
        let intervals = intervals.to_vec();
        let mut offsets = vec![0];
        offsets.reserve_exact(intervals.len());
        let mut size = 0;
        for (u, v) in &intervals {
            size += *v - *u + 1;
            offsets.push(size)
        }
        Self {
            intervals,
            offsets,
            size,
        }
    }

    pub fn get(&self, index: u32) -> u32 {
        // TODO. Should negative values be allowed?
        let mut j = self.intervals.len() - 1;
        if self.offsets[j] > index {
            let (mut hi, mut lo) = (j, 0usize);
            while lo + 1 < hi {
                let mid = (lo + hi) / 2;
                if self.offsets[mid] <= index {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            j = lo;
        }
        let t = index - self.offsets[j];
        let (u, _) = self.intervals[j];
        u + t
    }

    pub fn index(&self, value: u32) -> Option<u32> {
        for (offset, (u, v)) in self.offsets.iter().zip(self.intervals.iter()) {
            if *u == value {
                return Some(*offset);
            } else if *u > value {
                return None;
            } else if value <= *v {
                return Some(*offset + (value - u));
            }
        }
        None
    }

    pub fn index_above(&self, value: u32) -> u32 {
        for (offset, (u, v)) in self.offsets.iter().zip(self.intervals.iter()) {
            if *u >= value {
                return *offset;
            } else if value <= *v {
                return *offset + (value - u);
            }
        }
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_not_present() {
        assert!(IntervalSet::new(&[(1, 1)]).index(0).is_none());
        assert!(IntervalSet::new(&[]).index(0).is_none());
    }

    #[test]
    fn index_above_is_index_if_present() {
        assert_eq!(IntervalSet::new(&[(1, 10)]).index_above(1), 0);
        assert_eq!(IntervalSet::new(&[(1, 10)]).index_above(2), 1);
    }

    #[test]
    fn index_above_is_length_if_higher() {
        assert_eq!(IntervalSet::new(&[(1, 10)]).index_above(100), 10);
    }
}
