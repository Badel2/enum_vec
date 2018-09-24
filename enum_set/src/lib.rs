#![deny(missing_docs)]
//! A set for enum variants
//!
//! It is implemented as a wrapper over the `bit-set` crate,
//! which provides a set for integers values. We use the `EnumLike` trait
//! from the `enum_like` crate which allows for a conversion between enum
//! variant and integer value.
//!
//! Since an `EnumSet` is a wrapper over a `BitSet`, and a `BitSet` is
//! a wrapper over a `BitVec`, looking at the assembly generated by this crate
//! should be interesting.
//!
//! For usage examples, check out
//! <https://github.com/Badel2/enum_vec/blob/master/example/src/enum_set.rs>
extern crate enum_like;
extern crate bit_set;

use enum_like::EnumLike;
use bit_set::BitSet;
use std::marker::PhantomData;
use std::iter::FromIterator;
use std::fmt;

/// A `BitSet` indexed by an `EnumLike` type.
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnumSet<E: EnumLike> {
    inner: BitSet,
    _phantom: PhantomData<E>,
}

impl<E: EnumLike> EnumSet<E> {
    /// Returns the inner `BitSet`.
    pub fn into_bit_set(self) -> BitSet {
        self.inner
    }

    /// Returns a reference to the inner `BitSet`.
    pub fn get_ref(&self) -> &BitSet {
        &self.inner
    }

    /// Constructs an `EnumSet` from a `BitSet`.
    pub fn from_bit_set(inner: BitSet) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Creates a new `EnumSet`.
    pub fn new() -> Self {
        Self {
            //inner: BitSet::with_capacity(E::NUM_VARIANTS),
            inner: BitSet::new(),
            _phantom: PhantomData,
        }
    }

    /// Attemps to minimalize the memory usage of the inner `BitSet`.
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    /// Iterator over each element in the set.
    pub fn iter(&self) -> WrapIter<E, bit_set::Iter<'_, u32>> {
        WrapIter::<E, _>::new(self.inner.iter())
    }

    /*
    // Alternative not using WrapIter
    /// Iterator over each element in the set
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=E> + 'a {
        self.inner.iter().map(|x| E::from_discr(x))
    }
    */

    /// Iterator over each element in `set || other`.
    pub fn union<'a>(&'a self, other: &'a Self) -> Union<'a, E> {
        WrapIter::<E, _>::new(self.inner.union(&other.inner))
    }

    /// Iterator over each element in `set && other`.
    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, E> {
        WrapIter::<E, _>::new(self.inner.intersection(&other.inner))
    }

    /// Iterator over each element in `set - other`.
    pub fn difference<'a>(&'a self, other: &'a Self) -> Difference<'a, E> {
        WrapIter::<E, _>::new(self.inner.difference(&other.inner))
    }

    /// Iterator over each element in `set ^ other`.
    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifference<'a, E> {
        WrapIter::<E, _>::new(self.inner.symmetric_difference(&other.inner))
    }

    /// Computes the union with the other set, in-place.
    pub fn union_with(&mut self, other: &Self) {
        self.inner.union_with(&other.inner)
    }

    /// Computes the intersection with the other set, in-place.
    pub fn intersect_with(&mut self, other: &Self) {
        self.inner.intersect_with(&other.inner)
    }

    /// Computes the difference with the other set, in-place.
    pub fn difference_with(&mut self, other: &Self) {
        self.inner.difference_with(&other.inner)
    }

    /// Computes the symmetric difference with the other set, in-place.
    pub fn symmetric_difference_with(&mut self, other: &Self) {
        self.inner.symmetric_difference_with(&other.inner)
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there are no elements in the set.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clears all elements in the set.
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Returns `true` if the set contains this value.
    pub fn contains(&self, value: E) -> bool {
        let d = value.to_discr();
        self.inner.contains(d)
    }

    /// Returns `true` if the set has no elements in common with `other`.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.inner.is_disjoint(&other.inner)
    }

    /// Returns `true` if the set has no elements in common with `other`.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.inner.is_subset(&other.inner)
    }

    /// Returns `true` if the set has no elements in common with `other`.
    pub fn is_superset(&self, other: &Self) -> bool {
        self.inner.is_superset(&other.inner)
    }

    /// Returns `true` if the value was not already present in the set
    pub fn insert(&mut self, value: E) -> bool {
        let d = value.to_discr();
        self.inner.insert(d)
    }

    /// Returns `true` if the value was already present in the set
    pub fn remove(&mut self, value: E) -> bool {
        let d = value.to_discr();
        self.inner.remove(d)
    }
}

impl<E: EnumLike> Default for EnumSet<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: EnumLike> FromIterator<E> for EnumSet<E> {
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let mut ret = Self::default();
        ret.extend(iter);
        ret
    }
}

impl<E: EnumLike> Extend<E> for EnumSet<E> {
    fn extend<I: IntoIterator<Item = E>>(&mut self, iter: I) {
        for i in iter {
            self.insert(i);
        }
    }
}

impl<E: EnumLike + fmt::Debug> fmt::Debug for EnumSet<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

// Iterators

/// Wraps an iterator from the `bit-set` crate, mapping the output from
/// `usize` to `E: EnumLike`.
#[derive(Debug)]
pub struct WrapIter<E: EnumLike, I: Iterator<Item=usize>> {
    inner: I,
    _phantom: PhantomData<E>,
}

/// Iterator over the `&EnumSet`
pub type Iter<'a, E> = WrapIter<E, bit_set::Iter<'a, u32>>;
/// Iterator over the `&EnumSet`
pub type Union<'a, E> = WrapIter<E, bit_set::Union<'a, u32>>;
/// Iterator over the `&EnumSet`
pub type Intersection<'a, E> = WrapIter<E, bit_set::Intersection<'a, u32>>;
/// Iterator over the `&EnumSet`
pub type Difference<'a, E> = WrapIter<E, bit_set::Difference<'a, u32>>;
/// Iterator over the `&EnumSet`
pub type SymmetricDifference<'a, E> = WrapIter<E, bit_set::SymmetricDifference<'a, u32>>;

impl<E: EnumLike, I: Iterator<Item=usize>> WrapIter<E, I> {
    fn new(inner: I) -> Self {
        Self { inner, _phantom: PhantomData }
    }
}

impl<E: EnumLike, I: Iterator<Item=usize>> Iterator for WrapIter<E, I> {
    type Item = E;
    fn next(&mut self) -> Option<E> {
        self.inner.next().map(|x| E::from_discr(x))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, E: EnumLike> IntoIterator for &'a EnumSet<E> {
    type Item = E;
    type IntoIter = WrapIter<E, bit_set::Iter<'a, u32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}


/*
/// Make all the other methods of `BitVec` available without having to
/// rewrite them.
/// Now that I think about it, this may be a bad idea.
impl<E: EnumLike> Deref for EnumSet<E> {
    type Target = BitSet;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<E: EnumLike> DerefMut for EnumSet<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum ABC {
        A,
        B,
        C,
    }

    unsafe impl EnumLike for ABC {
        const NUM_VARIANTS: usize = 3;
        fn to_discr(self) -> usize {
            //self as usize
            // ^this may not work if the enum has variants with values, like A = 100:
            // so instead, we do the long form:
            match self {
                ABC::A => 0,
                ABC::B => 1,
                ABC::C => 2,
            }
        }
        fn from_discr(x: usize) -> Self {
            match x {
                0 => ABC::A,
                1 => ABC::B,
                2 => ABC::C,
                _ => panic!("Enum ABC has no variant {}", x),
            }
        }
    }

    #[test]
    fn create() {
        let mut e = EnumSet::new();
        assert_eq!(e.contains(ABC::A), false);
        assert_eq!(e.insert(ABC::A), true);
        assert_eq!(e.insert(ABC::A), false);
        assert_eq!(e.contains(ABC::A), true);
        assert_eq!(e.remove(ABC::A), true);
        assert_eq!(e.remove(ABC::A), false);
        assert_eq!(e.contains(ABC::A), false);
    }

    // As for now, this crate is assumed to work because of its simplicity.
}
