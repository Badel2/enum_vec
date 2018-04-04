#![deny(missing_docs)]

//! A vector which efficiently stores enum variants.

extern crate enum_like;

/// Not sure if this is needed
pub use enum_like::*;

use std::marker::PhantomData;

// Idea for SmallEnumVec: literally copy paste the code
// s/Vec/SmallVec
// Or maybe
// storage: union { Vec<u32>, [usize; 3] }
// Tag: MSB of num_elements

/// A vector which efficiently stores enum variants.
#[derive(Debug, Clone)]
pub struct EnumVec<T: EnumLike> {
    // The contents of the storage are undefined, even its length may not
    // match the expected self.len() / ELEMS_PER_BLOCK
    // TODO: generic over storage:
    // Vec<u8>, Vec<u16>, Box<[u8]>, Box<[u16]>, SmallVec<[u8; N]>, etc
    // RawVec isn't stable, but using a Box<[u32]> should be a better option
    // overhead: vec: 3 usize, box: 2 usize
    storage: Vec<u32>,
    num_elements: usize,
    phantom: PhantomData<T>,
}

#[allow(missing_docs)]
impl<T: EnumLike> EnumVec<T> {
    // Hacks, because const fn in not stable yet
    // FIXME: extract to
    // const fn bits_needed_for(NUM_VARIANTS) -> usize {
    //     usize_bits - NUM_VARIANTS.leading_zeros()
    // }
    // Also, this isn't always a power of two
    /// How many bits are needed to store a variant
    const BITS_PER_ELEM: usize = (T::NUM_VARIANTS > (1 << 0)) as usize
        + (T::NUM_VARIANTS > (1 << 1)) as usize
        + (T::NUM_VARIANTS > (1 << 2)) as usize
        + (T::NUM_VARIANTS > (1 << 3)) as usize
        + (T::NUM_VARIANTS > (1 << 4)) as usize
        + (T::NUM_VARIANTS > (1 << 5)) as usize
        + (T::NUM_VARIANTS > (1 << 6)) as usize
        + (T::NUM_VARIANTS > (1 << 7)) as usize
        + (T::NUM_VARIANTS > (1 << 8)) as usize
        + (T::NUM_VARIANTS > (1 << 9)) as usize
        + (T::NUM_VARIANTS > (1 << 10)) as usize
        + (T::NUM_VARIANTS > (1 << 11)) as usize
        + (T::NUM_VARIANTS > (1 << 12)) as usize
        + (T::NUM_VARIANTS > (1 << 13)) as usize
        + (T::NUM_VARIANTS > (1 << 14)) as usize
        + (T::NUM_VARIANTS > (1 << 15)) as usize
        + (T::NUM_VARIANTS > (1 << 16)) as usize
        + (T::NUM_VARIANTS > (1 << 17)) as usize
        + (T::NUM_VARIANTS > (1 << 18)) as usize
        + (T::NUM_VARIANTS > (1 << 19)) as usize
        + (T::NUM_VARIANTS > (1 << 20)) as usize
        + (T::NUM_VARIANTS > (1 << 21)) as usize
        + (T::NUM_VARIANTS > (1 << 22)) as usize
        + (T::NUM_VARIANTS > (1 << 23)) as usize
        + (T::NUM_VARIANTS > (1 << 24)) as usize
        + (T::NUM_VARIANTS > (1 << 25)) as usize
        + (T::NUM_VARIANTS > (1 << 26)) as usize
        + (T::NUM_VARIANTS > (1 << 27)) as usize
        + (T::NUM_VARIANTS > (1 << 28)) as usize
        + (T::NUM_VARIANTS > (1 << 29)) as usize
        + (T::NUM_VARIANTS > (1 << 30)) as usize
        + (T::NUM_VARIANTS > (1 << 31)) as usize
        + Self::ERROR_TOO_MANY_VARIANTS
        + Self::ERROR_ZERO_SIZED;

    const ERROR_TOO_MANY_VARIANTS: usize = 0
        // Error: cannot use EnumVec for 2^32 variants (because of 32 bit storage)
        - ((T::NUM_VARIANTS as u64 >= (1 << 32) ) as usize);

    // We could force zero sized types to use 1 bit, but that would be a waste
    const ERROR_ZERO_SIZED: usize = 0
        // Error: cannot use EnumVec for zero-sized types
        - ((T::NUM_VARIANTS <= 1) as usize);

    // How to handle zero sized types?
    //const ZERO_SIZED: bool = Self::BITS_PER_ELEM == 0;

    const ELEMS_PER_BLOCK: usize = 32 / Self::BITS_PER_ELEM;
    const ELEMENT_MASK: u32 = (1 << Self::BITS_PER_ELEM) - 1;
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    pub fn with_capacity(n: usize) -> Self {
        Self {
            storage: Vec::with_capacity(Self::blocks_for_elements(n)),
            num_elements: 0,
            phantom: PhantomData,
        }
    }
    pub fn capacity(&self) -> usize {
        self.storage
            .capacity()
            .saturating_mul(Self::ELEMS_PER_BLOCK)
    }
    pub fn get(&self, i: usize) -> Option<T> {
        // I wonder if T::from_discr will be optimized away when possible?
        self.get_raw(i).map(|x| T::from_discr(x))
    }
    pub fn set(&mut self, i: usize, x: T) {
        self.set_raw(i, x.to_discr());
    }
    pub fn push(&mut self, x: T) {
        self.grow_if_needed();
        let idx = self.num_elements;
        // max len is usize::MAX
        self.num_elements = self.num_elements
            .checked_add(1)
            .expect("capacity overflow");
        self.set(idx, x);
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let x = self.get(self.num_elements - 1).unwrap();
            self.num_elements -= 1;

            Some(x)
        }
    }
    pub fn reserve(&mut self, additional: usize) {
        let desired_cap = self.len()
            .checked_add(additional)
            .expect("capacity overflow");
        if desired_cap > self.capacity() {
            // Optimistically reserve more than is needed
            self.storage
                .reserve(1 + additional / Self::ELEMS_PER_BLOCK);
        }
    }
    pub fn shrink_to_fit(&mut self) {
        self.fix_storage();
        self.storage.shrink_to_fit();
    }
    pub fn truncate(&mut self, len: usize) {
        // Should this function truncate the storage?
        // Currently it doesn't, and we must use self.fix_storage()
        if len < self.num_elements {
            unsafe {
                self.set_len(len);
            }
        }
    }
    pub fn swap_remove(&mut self, index: usize) -> T {
        let length = self.len();
        self.swap(index, length - 1);

        self.pop().unwrap()
    }
    /// Insert and remove need a better implementation
    pub fn insert(&mut self, index: usize, element: T) {
        // Sorry, I was lazy, we just push and bubblesort the element into the
        // desired position
        let mut i = self.len();
        self.push(element);
        while i > index {
            self.swap(i - 1, i);
            i -= 1;
        }
    }
    pub fn remove(&mut self, index: usize) -> T {
        let x = self.get(index).unwrap();
        let mut i = index;
        let length = self.len() - 1;
        while i < length {
            let next = self.get_raw(i + 1).unwrap();
            self.set_raw(i, next);
            i += 1;
        }
        self.num_elements -= 1;

        x
    }
    pub fn append(&mut self, other: &mut Self) {
        let other_len = other.len();
        let self_len = self.len();
        if self.len() % Self::ELEMS_PER_BLOCK == 0 {
            // If the last block is full, we can just append the raw
            // representation
            // But first, we must fix the storage because its len may be bigger
            // than necessary
            self.fix_storage();
            other.fix_storage();
            self.storage.append(&mut other.storage);
            self.num_elements += other_len;
        } else {
            // Otherwise, just push every element
            self.reserve(other_len);
            self.num_elements += other_len;
            for i in 0..other_len {
                self.set_raw(self_len + i, other.get_raw(i).unwrap());
            }
        }
    }
    pub fn clear(&mut self) {
        // This doesn't actually clear anything, it justs sets the len to 0
        self.truncate(0);
    }
    pub fn len(&self) -> usize {
        self.num_elements
    }
    pub unsafe fn set_len(&mut self, len: usize) {
        self.num_elements = len;
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(at <= self.len(), "`at` out of bounds");
        // If at is the first element of a block, we could split_off the
        // internal storage instead, but we don't do that yet

        let other_len = self.len() - at;
        let mut other = Self::with_capacity(other_len);
        for i in 0..other_len {
            other.set_raw(i, self.get_raw(at + i).unwrap());
        }
        self.truncate(at);

        other
    }

    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        let len = self.len();

        if new_len > len {
            self.extend(std::iter::repeat(value).take(new_len - len));
        } else {
            self.truncate(new_len);
        }
    }

    fn get_raw(&self, i: usize) -> Option<usize> {
        if i >= self.len() {
            return None;
        }

        let discr = unsafe { self.get_raw_unchecked(i) };

        Some(discr)
    }
    /// Get the raw discriminant without bounds checking
    pub unsafe fn get_raw_unchecked(&self, i: usize) -> usize {
        let (idx_w, idx_b) = Self::block_index(i);
        let block = self.storage.get_unchecked(idx_w);
        let discr = (block >> idx_b) & Self::ELEMENT_MASK;

        discr as usize
    }
    fn set_raw(&mut self, i: usize, discr: usize) {
        if i >= self.len() {
            panic!("index out of bounds: {} >= {}", i, self.len());
        }

        unsafe {
            self.set_raw_unchecked(i, discr);
        }
    }
    /// Set the raw discriminant without bounds checking. It is assumed that
    /// the discriminant has only `Self::BITS_PER_ELEM` bits set.
    pub unsafe fn set_raw_unchecked(&mut self, i: usize, discr: usize) {
        let (idx_w, idx_b) = Self::block_index(i);
        let block = self.storage.get_unchecked_mut(idx_w);
        *block &= !(Self::ELEMENT_MASK << idx_b);
        *block |= (discr as u32) << idx_b;

        // Alternative implementation, TODO: benchmark
        /*
        let discr_old = (*block >> idx_b) & Self::ELEMENT_MASK;
        *block ^= (discr_old ^ discr as u32) << idx_b; 
        */    }
    fn swap(&mut self, ia: usize, ib: usize) {
        let a = self.get_raw(ia).unwrap();
        let b = self.get_raw(ib).unwrap();
        self.set_raw(ia, b);
        self.set_raw(ib, a);
    }
    fn grow_if_needed(&mut self) {
        // Grow if needed
        if (self.len() % Self::ELEMS_PER_BLOCK == 0)
            && (Self::blocks_for_elements(self.len()) == self.storage.len())
        {
            self.storage.push(0);
        }
    }
    // self.storage.len does never decrease, so here we fix it
    fn fix_storage(&mut self) {
        let len = Self::blocks_for_elements(self.len());
        self.storage.truncate(len);
    }
    // returns pair: (block, bit offset inside block)
    // bit offset means bit shift left
    // use ((self.storage[block] >> bit_offset) & ELEMENT_MASK) to get the value
    fn block_index(i: usize) -> (usize, usize) {
        (
            i / Self::ELEMS_PER_BLOCK,
            (i % Self::ELEMS_PER_BLOCK) * Self::BITS_PER_ELEM,
        )
    }
    fn blocks_for_elements(n: usize) -> usize {
        n.saturating_add(Self::ELEMS_PER_BLOCK - 1) / Self::ELEMS_PER_BLOCK
    }
    pub fn iter<'a>(&'a self) -> EnumVecIter<'a, T> {
        (&self).into_iter()
    }
    /*
    pub fn iter_mut<'a>(&'a mut self) -> EnumVecIterMut<'a, T> {
        (&mut self).into_iter()
    }
    */
    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        let l = self.len();
        for i in 0..l {
            let mut x = self.get(i).unwrap();
            f(&mut x);
            self.set(i, x);
        }
    }
}

impl<T: EnumLike> Extend<T> for EnumVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        // We could resize and use set_raw_unchecked, for speed
        for elem in iter {
            self.push(elem);
        }
    }
}

/*
//From and Into are just extend
impl<T: EnumLike> From<Vec<T>> for EnumVec<T> {
    fn from(vec: Vec<T>) -> Self {
        let mut e = EnumVec::new();
        e.extend(vec);

        e
    }
}

impl<T: EnumLike> Into<Vec<T>> for EnumVec<T> {
    fn into(self) -> Vec<T> {
        let mut v = Vec::new();
        v.extend(self);

        v
    }
}
*/

impl<'a, T: EnumLike> IntoIterator for &'a EnumVec<T> {
    type Item = T;
    type IntoIter = EnumVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        EnumVecIter {
            v: &self,
            idx: 0,
        }
    }
}

impl<T: EnumLike> IntoIterator for EnumVec<T> {
    type Item = T;
    type IntoIter = EnumVecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        EnumVecIntoIter {
            v: self,
            idx: 0,
        }
    }
}

// TODO: implement more iterator methods
// .iter() and .iter_mut() are implemented for slices,
// we can't return a slice, or a reference, just the value

/*
/// Iterator over &mut EnumVec
pub struct EnumVecIterMut<'a, T: 'a + EnumLike> {
    v: &'a mut EnumVec<T>,
    idx: usize,
}

impl<'a, T: EnumLike> Iterator for EnumVecIterMut<'a, T> {
    type Item = MutHack<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.idx;

        self.v.get(i).map(|x| {
            self.idx += 1;
            MutHack {
                value: x,
                vec: &mut self.v.storage,
                idx: i,
            }
        })
    }
}
*/

/// Iterator over &EnumVec
pub struct EnumVecIter<'a, T: 'a + EnumLike> {
    v: &'a EnumVec<T>,
    idx: usize,
}

impl<'a, T: EnumLike> Iterator for EnumVecIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.idx;

        self.v.get(i).map(|x| {
            self.idx += 1;
            x
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.v.len() - self.idx;

        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        let remaining = self.v.len() - self.idx;

        remaining
    }

    fn last(mut self) -> Option<Self::Item> {
        let v_len = self.v.len();
        if v_len == 0 || v_len == self.idx {
            None
        } else {
            self.idx = v_len - 1;

            self.next()
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.idx + n < self.v.len() {
            self.idx += n;

            self.next()
        } else {
            self.idx = self.v.len();

            None
        }
    }
}

//impl<T: EnumLike> ExactSizeIterator for EnumVecIter<T> {}

// Maybe we can implement it as EnumVecIter?
/*
pub struct EnumVecIntoIter<T: EnumLike> {
    inner: EnumVecIter<'self, T>,
    v: EnumVec<T>,
}
*/
/// Iterator over EnumVec
pub struct EnumVecIntoIter<T: EnumLike> {
    v: EnumVec<T>,
    idx: usize,
}

impl<T: EnumLike> Iterator for EnumVecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.idx;

        self.v.get(i).map(|x| {
            self.idx += 1;
            x
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.v.len() - self.idx;

        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        let remaining = self.v.len() - self.idx;

        remaining
    }

    fn last(mut self) -> Option<Self::Item> {
        let v_len = self.v.len();
        if v_len == 0 || v_len == self.idx {
            None
        } else {
            self.idx = v_len - 1;

            self.next()
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.idx + n < self.v.len() {
            self.idx += n;

            self.next()
        } else {
            self.idx = self.v.len();

            None
        }
    }
}

impl<T: EnumLike> ExactSizeIterator for EnumVecIntoIter<T> {}

/*
// This is a bad idea 
use std::ops::Rem;
impl<T: EnumLike> Rem<usize> for EnumVec<T> {
    type Output = T;

    fn rem(self, rhs: usize) -> Self::Output {
        self.get(rhs).expect("index out of range")
    }
}
*/

// Other useful impls:
impl<T: EnumLike> Default for EnumVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Warning: when implementing hash we must zero out the last block
// of the storage, otherwise the garbage data will make the hash inconsistent.
// Also, if we want to be generic over storage, we can't use the fast method
// of hashing each block, we must hash each element individually...

// Useful alias?
/// Alias for `EnumVec<bool>`
pub type BitVec = EnumVec<bool>;
// N-bit vec (currently unimplemented)
// needs const generics
// (and impl EnumLike for [bool; N])
//pub type NBitVec<N> = EnumVec<[bool; N]>;

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
    fn abc_push_pop() {
        let mut v = EnumVec::new();
        v.push(ABC::A);
        v.push(ABC::B);
        v.push(ABC::C);
        v.push(ABC::A);
        //panic!("Success! {:?}", v);

        assert_eq!(v.pop().unwrap(), ABC::A);
        assert_eq!(v.pop().unwrap(), ABC::C);
        assert_eq!(v.pop().unwrap(), ABC::B);
        assert_eq!(v.pop().unwrap(), ABC::A);
        assert!(v.pop().is_none());
    }

    #[test]
    fn option_abc() {
        let mut v = EnumVec::new();
        v.push(None);
        v.push(Some(ABC::A));
        v.push(Some(ABC::B));
        v.push(Some(ABC::C));
        v.push(Some(ABC::A));
        //panic!("Success! {:?}", v);

        assert_eq!(v.pop().unwrap().unwrap(), ABC::A);
        assert_eq!(v.pop().unwrap().unwrap(), ABC::C);
        assert_eq!(v.pop().unwrap().unwrap(), ABC::B);
        assert_eq!(v.pop().unwrap().unwrap(), ABC::A);
        assert_eq!(v.pop().unwrap(), None);
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn get_set() {
        let mut v = EnumVec::new();
        for _ in 0..10 {
            v.push(ABC::A);
        }

        v.set(3, ABC::C);
        v.set(5, ABC::B);

        for i in 0..10 {
            let expected = match i {
                3 => ABC::C,
                5 => ABC::B,
                _ => ABC::A,
            };

            assert_eq!(v.get(i).unwrap(), expected);
        }
    }

    #[test]
    fn insert_remove() {
        let mut v = EnumVec::new();
        for _ in 0..10 {
            v.push(ABC::B);
        }

        v.insert(0, ABC::C);
        v.insert(0, ABC::C);
        v.insert(0, ABC::C);
        v.insert(0, ABC::C);
        v.insert(0, ABC::C);

        v.insert(10, ABC::A);
        v.insert(10, ABC::A);
        v.insert(10, ABC::A);
        v.insert(10, ABC::A);
        v.insert(10, ABC::A);

        for i in 0..5 {
            assert_eq!(v.get(i).unwrap(), ABC::C);
        }
        for i in 5..10 {
            assert_eq!(v.get(i).unwrap(), ABC::B);
        }
        for i in 10..15 {
            assert_eq!(v.get(i).unwrap(), ABC::A);
        }
        for i in 15..20 {
            assert_eq!(v.get(i).unwrap(), ABC::B);
        }

        v.remove(0);
        v.remove(0);
        v.remove(0);
        v.remove(0);
        v.remove(0);
        v.insert(0, ABC::C);
        v.insert(0, ABC::C);
        v.insert(0, ABC::C);
        v.insert(0, ABC::C);
        v.insert(0, ABC::C);
        v.remove(15);
        v.remove(15);
        v.remove(15);
        v.remove(15);
        v.remove(15);
        v.insert(15, ABC::B);
        v.insert(15, ABC::B);
        v.insert(15, ABC::B);
        v.insert(15, ABC::B);
        v.insert(15, ABC::B);

        for i in 0..5 {
            assert_eq!(v.get(i).unwrap(), ABC::C);
        }
        for i in 5..10 {
            assert_eq!(v.get(i).unwrap(), ABC::B);
        }
        for i in 10..15 {
            assert_eq!(v.get(i).unwrap(), ABC::A);
        }
        for i in 15..20 {
            assert_eq!(v.get(i).unwrap(), ABC::B);
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct Digit {
        x: u8, // x >= 0 && x <= 9
    }

    unsafe impl EnumLike for Digit {
        const NUM_VARIANTS: usize = 10;
        fn to_discr(self) -> usize {
            self.x as usize
        }
        fn from_discr(x: usize) -> Self {
            let x = x as u8;
            Self { x }
        }
    }

    #[test]
    fn digit_test() {
        let mut v = EnumVec::new();
        // 9 digits so we must use two 32-bit blocks
        v.push(Digit { x: 3 });
        v.push(Digit { x: 4 });
        v.push(Digit { x: 5 });
        v.push(Digit { x: 6 });
        v.push(Digit { x: 3 });
        v.push(Digit { x: 4 });
        v.push(Digit { x: 5 });
        v.push(Digit { x: 6 });
        v.push(Digit { x: 3 });

        assert_eq!(v.pop().unwrap().x, 3);
        assert_eq!(v.pop().unwrap().x, 6);
        assert_eq!(v.pop().unwrap().x, 5);
        assert_eq!(v.pop().unwrap().x, 4);
        assert_eq!(v.pop().unwrap().x, 3);
        assert_eq!(v.pop().unwrap().x, 6);
        assert_eq!(v.pop().unwrap().x, 5);
        assert_eq!(v.pop().unwrap().x, 4);
        assert_eq!(v.pop().unwrap().x, 3);
        assert!(v.pop().is_none());
    }

    #[test]
    fn digit_uses_4_bits() {
        // Digit has NUM_VARIANTS = 10, so it uses 4 bits.
        // Therefore if we pass a digit with an invalid value,
        // it will only store the 4 least significant bits (LSB).
        // But it can override other elements in the vector,
        // so please don't do it.
        let mut v = EnumVec::new();
        v.push(Digit { x: 0x3B });
        let got = v.pop().unwrap().x;
        assert_eq!(got, 0x0B);
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct TwoDigits {
        tens: Digit,
        ones: Digit,
    }

    unsafe impl EnumLike for TwoDigits {
        const NUM_VARIANTS: usize = Digit::NUM_VARIANTS * Digit::NUM_VARIANTS;
        fn to_discr(self) -> usize {
            self.tens.to_discr() * Digit::NUM_VARIANTS + self.ones.to_discr()
        }
        fn from_discr(x: usize) -> Self {
            let tens = Digit::from_discr(x / Digit::NUM_VARIANTS);
            let ones = Digit::from_discr(x % Digit::NUM_VARIANTS);

            Self { tens, ones }
        }
    }

    #[test]
    fn two_digit_test() {
        let mut ev = EnumVec::new();
        let mut v = Vec::new();
        for i in 0..10 {
            let d = TwoDigits {
                tens: Digit { x: i },
                ones: Digit { x: 9 - i },
            };
            ev.push(d);
            v.push(d);
        }

        for i in 0..10 {
            assert_eq!((i, v[i]), (i, ev.get(i).unwrap()));
        }

        for i in 0..10 {
            let mut d = ev.get(i).unwrap();
            d.tens.x = (d.tens.x + 3) % 10;
            d.ones.x = (d.ones.x * 2 + 1) % 10;
            ev.set(i, d);
            // Just so we test something
            assert_eq!(d, ev.get(i).unwrap());
        }
    }

    #[test]
    fn from_vec() {
        let a = vec![ABC::C, ABC::A, ABC::A, ABC::B, ABC::C];
        let mut v = EnumVec::new();
        v.extend(a.clone());
        for i in 0..a.len() {
            assert_eq!(a[i], v.get(i).unwrap());
        }
    }

    /*
    // It doesn't even compile
    #[test]
    fn zero_sized() {
        let _v: EnumVec<()> = EnumVec::new();
    }
    */
}
