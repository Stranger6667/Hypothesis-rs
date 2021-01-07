use crate::Error;
use bitmaps::Bitmap;
use core::fmt;
use smallvec::alloc::fmt::Formatter;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::{BitAnd, BitOr, BitXor};
use typenum::U30;

pub(crate) const ALL_CATEGORIES: u32 = 0x3fffffff;

#[allow(non_snake_case)]
pub mod categories {
    use crate::CategoryBitMap;
    use crate::UnicodeCategory::*;

    #[inline]
    pub fn L() -> CategoryBitMap {
        Ll | Lm | Lo | Lt | Lu
    }
    #[inline]
    pub fn M() -> CategoryBitMap {
        Mc | Me | Mn
    }
    #[inline]
    pub fn N() -> CategoryBitMap {
        Nd | Nl | No
    }
    #[inline]
    pub fn P() -> CategoryBitMap {
        Pc | Pd | Pe | Pf | Pi | Po | Ps
    }
    #[inline]
    pub fn S() -> CategoryBitMap {
        Sc | Sk | Sm | So
    }
    #[inline]
    pub fn Z() -> CategoryBitMap {
        Zp | Zs | Zl
    }
    #[inline]
    pub fn C() -> CategoryBitMap {
        Cc | Cf | Cn | Co | Cs
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum UnicodeCategory {
    Pe = 0,
    Pc = 1,
    Cc = 2,
    Sc = 3,
    Pd = 4,
    Nd = 5,
    Me = 6,
    Pf = 7,
    Cf = 8,
    Pi = 9,
    Nl = 10,
    Zl = 11,
    Ll = 12,
    Sm = 13,
    Lm = 14,
    Sk = 15,
    Mn = 16,
    Ps = 17,
    Lo = 18,
    No = 19,
    Po = 20,
    So = 21,
    Zp = 22,
    Co = 23,
    Zs = 24,
    Mc = 25,
    Cs = 26,
    Lt = 27,
    Cn = 28,
    Lu = 29,
}

impl BitOr for UnicodeCategory {
    type Output = CategoryBitMap;

    fn bitor(self, rhs: Self) -> Self::Output {
        CategoryBitMap::from([self, rhs].as_ref())
    }
}

impl<'a> TryFrom<&'a str> for UnicodeCategory {
    type Error = Error<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Pe" => Self::Pe,
            "Pc" => Self::Pc,
            "Cc" => Self::Cc,
            "Sc" => Self::Sc,
            "Pd" => Self::Pd,
            "Nd" => Self::Nd,
            "Me" => Self::Me,
            "Pf" => Self::Pf,
            "Cf" => Self::Cf,
            "Pi" => Self::Pi,
            "Nl" => Self::Nl,
            "Zl" => Self::Zl,
            "Ll" => Self::Ll,
            "Sm" => Self::Sm,
            "Lm" => Self::Lm,
            "Sk" => Self::Sk,
            "Mn" => Self::Mn,
            "Ps" => Self::Ps,
            "Lo" => Self::Lo,
            "No" => Self::No,
            "Po" => Self::Po,
            "So" => Self::So,
            "Zp" => Self::Zp,
            "Co" => Self::Co,
            "Zs" => Self::Zs,
            "Mc" => Self::Mc,
            "Cs" => Self::Cs,
            "Lt" => Self::Lt,
            "Cn" => Self::Cn,
            "Lu" => Self::Lu,
            _ => return Err(Self::Error::InvalidCategory(value)),
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CategoryBitMap(Bitmap<U30>);

impl Hash for CategoryBitMap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.0.into_value())
    }
}

impl PartialEq for CategoryBitMap {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for CategoryBitMap {}

impl Default for CategoryBitMap {
    fn default() -> Self {
        let map: Bitmap<U30> = Bitmap::new();
        Self(map)
    }
}

impl CategoryBitMap {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from_value(value: u32) -> Self {
        Self(Bitmap::from_value(value))
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> Iter {
        Iter {
            len: self.0.len(),
            inner: self.0.into_iter(),
        }
    }

    #[inline]
    pub fn to_vec_str(self) -> Vec<&'static str> {
        self.iter().collect()
    }

    #[inline]
    pub fn first_index(self) -> Option<usize> {
        self.0.first_index()
    }

    #[inline]
    pub fn add_category(&mut self, category: UnicodeCategory) {
        self.0.set(category as usize, true);
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: bool) -> bool {
        self.0.set(index, value)
    }

    #[inline]
    pub fn try_extend<'a>(&mut self, value: &'a str) -> Result<(), Error<'a>> {
        match value {
            "L" => {
                for variant in &[
                    UnicodeCategory::Ll,
                    UnicodeCategory::Lm,
                    UnicodeCategory::Lo,
                    UnicodeCategory::Lt,
                    UnicodeCategory::Lu,
                ] {
                    self.add_category(*variant)
                }
            }
            "M" => {
                for variant in &[
                    UnicodeCategory::Mc,
                    UnicodeCategory::Me,
                    UnicodeCategory::Mn,
                ] {
                    self.add_category(*variant)
                }
            }
            "N" => {
                for variant in &[
                    UnicodeCategory::Nd,
                    UnicodeCategory::Nl,
                    UnicodeCategory::No,
                ] {
                    self.add_category(*variant)
                }
            }
            "P" => {
                for variant in &[
                    UnicodeCategory::Pc,
                    UnicodeCategory::Pd,
                    UnicodeCategory::Pe,
                    UnicodeCategory::Pf,
                    UnicodeCategory::Pi,
                    UnicodeCategory::Po,
                    UnicodeCategory::Ps,
                ] {
                    self.add_category(*variant)
                }
            }
            "S" => {
                for variant in &[
                    UnicodeCategory::Sc,
                    UnicodeCategory::Sk,
                    UnicodeCategory::Sm,
                    UnicodeCategory::So,
                ] {
                    self.add_category(*variant)
                }
            }
            "Z" => {
                for variant in &[
                    UnicodeCategory::Zl,
                    UnicodeCategory::Zp,
                    UnicodeCategory::Zs,
                ] {
                    self.add_category(*variant)
                }
            }
            "C" => {
                for variant in &[
                    UnicodeCategory::Cc,
                    UnicodeCategory::Cf,
                    UnicodeCategory::Cn,
                    UnicodeCategory::Co,
                    UnicodeCategory::Cs,
                ] {
                    self.add_category(*variant)
                }
            }
            other => {
                let category = UnicodeCategory::try_from(other)?;
                self.add_category(category)
            }
        }
        Ok(())
    }
}

impl fmt::Display for CategoryBitMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let iter = self.iter();
        let len = iter.len();
        for (idx, cat) in iter.enumerate() {
            f.write_str(cat)?;
            if idx + 1 != len {
                f.write_str(", ")?;
            }
        }
        Ok(())
    }
}

#[allow(missing_debug_implementations)]
pub struct Iter<'a> {
    len: usize,
    inner: bitmaps::Iter<'a, U30>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'static str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(variant) = self.inner.next() {
            match variant {
                x if x == UnicodeCategory::Pe as usize => Some("Pe"),
                x if x == UnicodeCategory::Pc as usize => Some("Pc"),
                x if x == UnicodeCategory::Cc as usize => Some("Cc"),
                x if x == UnicodeCategory::Sc as usize => Some("Sc"),
                x if x == UnicodeCategory::Pd as usize => Some("Pd"),
                x if x == UnicodeCategory::Nd as usize => Some("Nd"),
                x if x == UnicodeCategory::Me as usize => Some("Me"),
                x if x == UnicodeCategory::Pf as usize => Some("Pf"),
                x if x == UnicodeCategory::Cf as usize => Some("Cf"),
                x if x == UnicodeCategory::Pi as usize => Some("Pi"),
                x if x == UnicodeCategory::Nl as usize => Some("Nl"),
                x if x == UnicodeCategory::Zl as usize => Some("Zl"),
                x if x == UnicodeCategory::Ll as usize => Some("Ll"),
                x if x == UnicodeCategory::Sm as usize => Some("Sm"),
                x if x == UnicodeCategory::Lm as usize => Some("Lm"),
                x if x == UnicodeCategory::Sk as usize => Some("Sk"),
                x if x == UnicodeCategory::Mn as usize => Some("Mn"),
                x if x == UnicodeCategory::Ps as usize => Some("Ps"),
                x if x == UnicodeCategory::Lo as usize => Some("Lo"),
                x if x == UnicodeCategory::No as usize => Some("No"),
                x if x == UnicodeCategory::Po as usize => Some("Po"),
                x if x == UnicodeCategory::So as usize => Some("So"),
                x if x == UnicodeCategory::Zp as usize => Some("Zp"),
                x if x == UnicodeCategory::Co as usize => Some("Co"),
                x if x == UnicodeCategory::Zs as usize => Some("Zs"),
                x if x == UnicodeCategory::Mc as usize => Some("Mc"),
                x if x == UnicodeCategory::Cs as usize => Some("Cs"),
                x if x == UnicodeCategory::Lt as usize => Some("Lt"),
                x if x == UnicodeCategory::Cn as usize => Some("Cn"),
                x if x == UnicodeCategory::Lu as usize => Some("Lu"),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.len
    }
}

impl BitOr<UnicodeCategory> for CategoryBitMap {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: UnicodeCategory) -> Self::Output {
        let mut new = self;
        new.add_category(rhs);
        new
    }
}

impl<'a> TryFrom<&[&'a str]> for CategoryBitMap {
    type Error = Error<'a>;

    #[inline]
    fn try_from(value: &[&'a str]) -> Result<Self, Self::Error> {
        let mut out = Self::new();
        for category in value.iter() {
            out.try_extend(category)?
        }
        Ok(out)
    }
}

impl<'a> TryFrom<&'a str> for CategoryBitMap {
    type Error = Error<'a>;

    #[inline]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut out = Self::new();
        out.try_extend(value)?;
        Ok(out)
    }
}

impl From<&[UnicodeCategory]> for CategoryBitMap {
    #[inline]
    fn from(values: &[UnicodeCategory]) -> Self {
        let mut out = Self::new();
        for value in values {
            out.add_category(*value)
        }
        out
    }
}
impl From<UnicodeCategory> for CategoryBitMap {
    #[inline]
    fn from(category: UnicodeCategory) -> Self {
        let mut out = Self::new();
        out.add_category(category);
        out
    }
}

impl BitOr for CategoryBitMap {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        CategoryBitMap(self.0 | rhs.0)
    }
}

impl BitAnd for CategoryBitMap {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        CategoryBitMap(self.0 & rhs.0)
    }
}

impl BitXor for CategoryBitMap {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        CategoryBitMap(self.0 ^ rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_are_combinable() {
        let categories = UnicodeCategory::Cs | UnicodeCategory::Zs | UnicodeCategory::Zl;
        assert_eq!(
            categories.iter().collect::<Vec<&str>>(),
            vec!["Zl", "Zs", "Cs"]
        )
    }

    #[test]
    fn categories_display() {
        let categories = UnicodeCategory::Cs | UnicodeCategory::Zs | UnicodeCategory::Zl;
        assert_eq!(format!("{}", categories), "Zl, Zs, Cs")
    }
}
