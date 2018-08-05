use enum_like::EnumLike;
use smallvec::SmallVec;
use std::fmt;
use std::iter::{FromIterator, repeat};
use std::marker::PhantomData;
use std::ops::Range;
use std::hash::{Hash, Hasher};
use std::cmp;

type StorageBlock = u32;
type Storage = SmallVec<[StorageBlock; 4]>;
const STORAGE_BLOCK_SIZE: usize = 32;

/// A vector which efficiently stores enum variants.
#[derive(Clone)]
pub struct EnumVec<T: EnumLike> {
    // The contents of the storage are undefined, even its length may not
    // match the expected self.len() / ELEMS_PER_BLOCK
    // TODO: generic over storage:
    // Vec<u8>, Vec<u16>, Box<[u8]>, Box<[u16]>, SmallVec<[u8; N]>, etc
    // RawVec isn't stable, but using a Box<[u32]> should be a better option
    // overhead: vec: 3 usize, box: 2 usize
    storage: Storage,
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

    const ERROR_TOO_MANY_VARIANTS: usize = 0 /*Error: this type has too many variants for this storage, try using enum_vec::vec_u64::EnumVec*/ - ((T::NUM_VARIANTS as u64 > (1 << STORAGE_BLOCK_SIZE) ) as usize);

    // We could force zero sized types to use 1 bit, but that would be a waste
    const ERROR_ZERO_SIZED: usize = 0
        // Error: cannot use EnumVec for zero-sized types
        - ((T::NUM_VARIANTS <= 1) as usize);

    // How to handle zero sized types?
    //const ZERO_SIZED: bool = Self::BITS_PER_ELEM == 0;

    const ELEMS_PER_BLOCK: usize = STORAGE_BLOCK_SIZE / Self::BITS_PER_ELEM;
    // While wrapping_shl is not const fn, we support at most 64 bits per element
    const ELEMENT_MASK: StorageBlock = ((1u64 << Self::BITS_PER_ELEM) - 1) as StorageBlock;
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_capacity(n: usize) -> Self {
        Self {
            storage: Storage::with_capacity(Self::blocks_for_elements(n)),
            num_elements: 0,
            phantom: PhantomData,
        }
    }
    /// Returns the number of elements that can be hold without
    /// allocating new memory.
    ///
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let ev = EnumVec::<bool>::with_capacity(53);
    /// assert!(ev.capacity() >= 53);
    /// ```
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
    /// Reserves capacity for at least `additional` more elements.
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut ev: EnumVec<Option<()>> = vec![None, None, None].into();
    /// ev.reserve(100);
    /// assert!(ev.capacity() >= 100 + 3);
    ///
    /// let mut a: EnumVec<bool> = EnumVec::new();
    /// a.reserve(1);
    /// assert_eq!(a.len(), 0);
    /// assert!(ev.storage().len() > 0);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        let desired_cap = self
            .len()
            .checked_add(additional)
            .expect("capacity overflow");
        // if desired_cap > self.capacity() {
            // Optimistically reserve more than is needed
            // And zero-out the storage, to enable get_raw and set_raw
            // TODO: should we? reserve is not meant to change the len
            // the problem is that set_len does not set the len of the
            // storage... but that would be too inefficient, it may be
            // better to just add calls to fix_storage() in clone()
            //let old_slen = self.storage.len();
        //}
        // Always resize storage until we can be sure that
        // self.storage.len() == self.storage.capacity()
        self.storage.resize(desired_cap / Self::ELEMS_PER_BLOCK + 1, 0);
    }
    /// Shrinks the capacity as much as possible.
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
    /// Remove an element from an arbitrary position in O(1) time,
    /// but without preserving the ordering.
    /// This is accomplished by swapping the desired element with
    /// the last element, and then calling `pop()`.
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut ev: EnumVec<bool> = vec![true, true, true, false, false].into();
    /// ev.swap_remove(0);
    /// assert_eq!(&ev.to_vec(), &[false, true, true, false]);
    /// ev.swap_remove(1);
    /// assert_eq!(&ev.to_vec(), &[false, false, true]);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        let length = self.len();
        self.swap(index, length - 1);

        self.pop().unwrap()
    }
    /// Insert an element into an arbitrary position. This operation is very
    /// expensive, as it must shift all the elements to make space for the new
    /// one. Prefer using `push()`.
    ///
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut ev = EnumVec::new();
    /// ev.insert(0, true);
    /// assert_eq!(ev.to_vec(), vec![true]);
    /// ev.insert(0, false);
    /// assert_eq!(ev.to_vec(), vec![false, true]);
    /// ev.insert(1, true);
    /// assert_eq!(ev.to_vec(), vec![false, true, true]);
    /// ev.insert(1, false);
    /// assert_eq!(ev.to_vec(), vec![false, false, true, true]);
    ///
    /// let mut ev: EnumVec<_> = vec![false; 127].into();
    /// assert_eq!(ev.len(), 127);
    /// ev.push(true);
    /// assert_eq!(ev.len(), 127 + 1);
    /// ev.push(true);
    /// assert_eq!(ev.len(), 127 + 2);
    /// ev.insert(0, true);
    /// ev.insert(0, true);
    /// assert_eq!((ev.get(0).unwrap(), ev.get(1).unwrap()), (true, true));
    /// assert_eq!(ev.len(), 127 + 4);
    /// ```
    pub fn insert(&mut self, index: usize, element: T) {
        let shift_storage = |block: &mut StorageBlock, at_zero: StorageBlock| {
            let last_bit_offset = (Self::ELEMS_PER_BLOCK - 1) * Self::BITS_PER_ELEM;
            let last = *block >> last_bit_offset & Self::ELEMENT_MASK;
            *block <<= Self::BITS_PER_ELEM;
            *block |= at_zero;

            last
        };

        // Increment the len by 1 and allocate memory if needed
        self.push(element);

        let slow_insert = index % Self::ELEMS_PER_BLOCK;
        let self_len = self.len();
        let mut prev = element.to_discr();
        let mut i = index;
        // Skip this part if index % Self::ELEMS_PER_BLOCK == 0
        if slow_insert > 0 {
            let slow_limit = index - slow_insert + Self::ELEMS_PER_BLOCK;
            let slow_limit = cmp::min(slow_limit, self_len);
            while i < slow_limit {
                unsafe {
                    // This is safe if we stay inside the same storage
                    // block, even if we wanted to access i > self.len()
                    let next = self.get_raw_unchecked(i); 
                    self.set_raw_unchecked(i, prev);
                    prev = next;
                }
                i += 1;
            }
        }

        if i < self.len() {
            // Shift the storage blocks, including the last block
            let (ib, _) = Self::block_index(i);
            let (last_ib, _) = Self::block_index(self.len() - 1);
            let mut prev = prev as StorageBlock;
            for i in ib..(last_ib + 1) {
                prev = shift_storage(&mut self.storage[i], prev);
            }
        }
    }
    /// Remove an element from an arbitrary position. This operation is very
    /// expensive, as it must shift all the elements to fill the hole.
    /// When preserving the order is not important, consider using
    /// `swap_remove()`.
    ///
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut ev: EnumVec<_> = vec![None; 129].into();
    /// let item = Some([true, false, true, false]);
    /// ev.insert(0, item);
    /// assert_eq!(ev.remove(0), item);
    /// ev.insert(127, item);
    /// assert_eq!(ev.remove(127), item);
    /// ev.insert(127, item);
    /// ev.insert(128, item);
    /// assert_eq!(ev.remove(127), item);
    /// assert_eq!(ev.remove(127), item);
    /// assert_eq!(ev.remove(127), None);
    /// assert_eq!(ev.remove(127), None);
    /// assert_eq!(ev.len(), 127);
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        let x = self.get(index).unwrap();

        let shift_storage = |block: &mut StorageBlock, at_zero: StorageBlock| {
            let last = *block & Self::ELEMENT_MASK;
            let end_bit_offset = (Self::ELEMS_PER_BLOCK - 1) * Self::BITS_PER_ELEM;
            *block >>= Self::BITS_PER_ELEM;
            *block |= at_zero << end_bit_offset;

            last
        };

        let mut i = index;
        let length = self.len() - 1;
        let block_limit = (1 + Self::block_index(i).0) * Self::ELEMS_PER_BLOCK;
        let slow_limit = cmp::min(length, block_limit);
        while i < slow_limit {
            unsafe { // safe: i + 1 < self.len()
                let next = self.get_raw_unchecked(i + 1);
                self.set_raw_unchecked(i, next);
            }
            i += 1;
        }

        let prev = unsafe {
            self.get_raw_unchecked(self.len() - 1)
        };

        self.num_elements -= 1;

        if i < self.len() {
            // Shift the storage blocks, including the last block
            let (ib, _) = Self::block_index(i);
            let (last_ib, _) = Self::block_index(self.len() - 1);
            let mut prev = prev as StorageBlock;
            for i in (ib..(last_ib + 1)).rev() {
                prev = shift_storage(&mut self.storage[i], prev);
            }
        }

        x
    }
    /// Retains only the elements specified by the predicate
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut v: EnumVec<(bool, bool)> = vec![(true, true), (false, false), (true, false),
    ///     (false, true)].into();
    /// v.retain(|x| x.0 == true);
    /// let a = v.to_vec();
    /// assert_eq!(&a, &[(true, true), (true, false)]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool
    {
        let mut i_get = 0;
        let mut i_set = 0;

        let l = self.len();
        while i_get < l {
            let x = self.get(i_get).unwrap();
            if f(&x) {
                self.set(i_set, x);
                i_set += 1;
            }
            i_get += 1;
        }

        unsafe {
            self.set_len(i_set);
        }
    }
    /// Push an element to the end of the vector.
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut ev: EnumVec<_> = vec![false; 127].into();
    /// ev.push(true);
    /// ```
    pub fn push(&mut self, x: T) {
        self.grow_if_needed();
        let idx = self.num_elements;
        // max len is usize::MAX
        self.num_elements =
            self.num_elements.checked_add(1).expect("capacity overflow");
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
    /// Appends all the elements from `other` into `self`, leaving
    /// `other` empty.
    /// This can be more efficient than using `extend` when the internal
    /// storage is block-aligned. For example when each element is 4-bits wide
    /// and the storage is a u32, it is block-aligned when it has 8*k elements.
    /// Also, it doesn't have to map all the elements from `T` to discr.
    ///
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut a: EnumVec<_> = vec![None, None, Some(())].into();
    /// let mut b = a.clone();
    /// b.push(None);
    /// a.append(&mut b);
    /// assert_eq!(a.get(6), Some(None));
    /// assert_eq!(a.len(), 3+4);
    /// assert_eq!(b.len(), 0);
    /// ```
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
            self.storage.extend_from_slice(&other.storage);
other.clear();
            self.num_elements += other_len;
        } else {
            // Otherwise, just push every element
            self.reserve(other_len);
            unsafe { // We just reserved space
                self.set_len(self_len + other_len);
                for i in 0..other_len {
                    self.set_raw_unchecked(self_len + i, other.get_raw_unchecked(i));
                }
            }

            other.clear();
        }
    }
    /// Sets the length to zero, removing all the elements.
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut ev = EnumVec::new();
    /// ev.push(Some(false));
    /// assert_eq!(ev.len(), 1);
    /// ev.clear();
    /// assert_eq!(ev.len(), 0);
    /// assert!(ev.is_empty());
    ///
    /// unsafe {
    ///     ev.set_len(1);
    ///     assert_eq!(ev.pop().unwrap(), Some(false));
    /// }
    /// ```
    pub fn clear(&mut self) {
        // This doesn't actually clear anything, it justs sets the len to 0
        self.truncate(0);
    }
    /// Returns the length of the vector, the number of elements it holds.
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
        unsafe {
            other.set_len(other_len);
            for i in 0..other_len {
                other.set_raw_unchecked(i, self.get_raw_unchecked(at + i));
            }
            self.set_len(at);
        }

        other
    }

    pub fn resize(&mut self, new_len: usize, value: T)
    {
        let len = self.len();

        if new_len > len {
            self.extend_with_value(value, new_len - len);
        } else {
            self.truncate(new_len);
        }
    }

    /// This is the equivalent to a memset
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut ev = EnumVec::from_elem((true, false), 1000);
    /// assert_eq!(ev.len(), 1000);
    /// assert!(ev.iter().all(|x| x == (true, false)));
    /// ```
    fn extend_with_value(&mut self, value: T, count: usize) {
        if count <= 4 * Self::ELEMS_PER_BLOCK {
            // Slow path, the overhead is not worth it
            self.extend(repeat(value).take(count));
        } else {
            // First fill out the last storage block:
            let to_insert_first = Self::ELEMS_PER_BLOCK -
                                (self.len() % Self::ELEMS_PER_BLOCK);
            self.extend(repeat(value).take(to_insert_first));
            let count_end = count - to_insert_first;
            let count_blocks = count_end / Self::ELEMS_PER_BLOCK;
            let to_insert_mid = count_blocks * Self::ELEMS_PER_BLOCK;
            let to_insert_last = count - (to_insert_first + to_insert_mid);

            let d = value.to_discr();
            let mut block_value = d as StorageBlock;
            let mut i = Self::BITS_PER_ELEM;

            // Assuming that STORAGE_BLOCK_SIZE is a power of 2
            while i < STORAGE_BLOCK_SIZE {
                block_value |= block_value << i;
                i *= 2;
            }

            let old_len = self.len();

            // Set storage len to self.len() / Self::ELEMS_PER_BLOCK,
            // so pushing to the enumvec is equivalent to pushing to storage
            self.fix_storage();
            self.storage.reserve(count_blocks + (to_insert_last > 0) as usize);

            // Now fill the itermediate blocks using memcpy, this is
            // the optimization
            for _ in 0..count_blocks {
                let storage_len = self.storage.len();
                self.storage.resize(storage_len + count_blocks, block_value);
            }

            unsafe {
                self.set_len(old_len + count_blocks * Self::ELEMS_PER_BLOCK);
            }

            // Finally fill out the remaining elements
            self.extend(repeat(value).take(to_insert_last));
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
    /// the discriminant is lower than `T::NUM_ELEMENTS`.
    pub unsafe fn set_raw_unchecked(&mut self, i: usize, discr: usize) {
        let (idx_w, idx_b) = Self::block_index(i);
        let block = self.storage.get_unchecked_mut(idx_w);
        *block &= !(Self::ELEMENT_MASK << idx_b);
        *block |= (discr as StorageBlock) << idx_b;

        // Alternative implementation, TODO: benchmark
        /*
        let discr_old = (*block >> idx_b) & Self::ELEMENT_MASK;
        *block ^= (discr_old ^ discr as StorageBlock) << idx_b; 
        */
    }

    /// Swap two elements.
    pub fn swap(&mut self, ia: usize, ib: usize) {
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

    // iter_mut cannot be implemented because we cannot take a reference to
    // the values inside the EnumVec. Use for_each() instead.
    /*
    pub fn iter_mut<'a>(&'a mut self) -> EnumVecIterMut<'a, T> {
        (&mut self).into_iter()
    }
    */
    /// Apply a function to each element in place, this is a substitute to
    /// for loops:
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut v = vec![true, false, true];
    /// for x in v.iter_mut() {
    ///     *x = !*x;
    /// }
    ///
    /// // Is equivalent to
    /// let mut ev: EnumVec<_> = vec![true, false, true].into();
    /// ev.for_each(|x| {
    ///     *x = !*x;
    /// });
    ///
    /// assert_eq!(v, ev.to_vec());
    /// assert_eq!(&v, &[false, true, false]);
    /// ```
    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        let l = self.len();
        for i in 0..l {
            let mut x = self.get(i).unwrap();
            f(&mut x);
            // if x changed?
            self.set(i, x);
        }
    }

    /// Copies `self` into a plain `Vec`.
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut ev = EnumVec::new();
    /// ev.push(true);
    /// ev.push(false);
    /// let v = vec![true, false];
    /// assert_eq!(ev.to_vec(), v);
    /// ```
    pub fn to_vec(&self) -> Vec<T> {
        let mut v = Vec::with_capacity(self.len());
        v.extend(self.iter());

        v
    }

    pub fn from_elem(x: T, n: usize) -> Self
    {
        let mut v = EnumVec::new();
        v.extend_with_value(x, n);

        v
    }

    /// ```
    /// #[macro_use] extern crate enum_vec;
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let ev = enum_vec![true, false, false, true];
    /// assert_eq!(ev.len(), 4);
    /// let mut a = enum_vec![false; 8];
    /// a.extend(ev);
    /// ```
    pub fn from_slice(x: &[T]) -> Self
    {
        let mut v = EnumVec::new();
        v.extend(x.iter().cloned());

        v
    }

    /// Access the internal storage
    /// ```
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let ev: EnumVec<_> = vec![true; 100].into();
    /// assert_eq!(ev.len(), 100);
    /// assert!(ev.storage().len() > 0);
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Access and modify the internal storage.
    /// This function is unsafe because shrinking the storage
    /// may lead to reading and writing uninitialized memory.
    /// ```
    /// extern crate enum_like;
    /// extern crate enum_vec;
    /// use enum_like::EnumLike;
    /// use enum_vec::smallvec_u32::EnumVec;
    ///
    /// let mut ev: EnumVec<_> = vec![true; 100].into();
    /// assert_eq!(ev.len(), 100);
    /// assert!(ev.storage().len() > 0);
    /// assert_eq!(ev.get(0).unwrap(), true);
    ///
    /// unsafe {
    ///     let s = ev.storage_mut();
    ///     // We set the first block to 0, which will set it to false:
    ///     assert_eq!(false.to_discr(), 0);
    ///     s[0] = 0;
    /// }
    ///
    /// // The number of modified elements depends on
    /// // the storage size and the element size, but the first element
    /// // will always be changed
    /// assert_eq!(ev.get(0).unwrap(), false);
    /// ```
    pub unsafe fn storage_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }
}

// TODO: impl Clone and fix storage

impl<T: EnumLike + fmt::Debug> fmt::Debug for EnumVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: EnumLike> Default for EnumVec<T> {
    fn default() -> Self {
        Self {
            storage: Storage::new(),
            num_elements: 0,
            phantom: PhantomData,
        }
    }
}

impl<T: EnumLike> Extend<T> for EnumVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        let iter_len = iter.size_hint().0;
        let self_len = self.len();
        // We could resize and use set_raw_unchecked, for speed
        self.reserve(iter_len);
        self.num_elements += iter_len;
        for i in 0..iter_len {
            self.set(self_len + i, iter.next().unwrap());
        }

        // Push the remaining elements, if any
        for elem in iter {
            self.push(elem);
        }
    }
}

impl<T: EnumLike> FromIterator<T> for EnumVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut v = EnumVec::new();
        v.extend(iter);

        v
    }
}

// Convenience impls to allow
// let e: EnumVec<_> = vec![A, B, C].into();
// and
// let v: Vec<_> = e.into();
impl<T: EnumLike> From<Vec<T>> for EnumVec<T> {
    fn from(v: Vec<T>) -> Self {
        EnumVec::from_iter(v)
    }
}

impl<T: EnumLike> Into<Vec<T>> for EnumVec<T> {
    fn into(self) -> Vec<T> {
        self.to_vec()
    }
}

impl<'a, T: EnumLike> IntoIterator for &'a EnumVec<T> {
    type Item = T;
    type IntoIter = EnumVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let l = self.len();
        EnumVecIter {
            v: &self,
            range: 0..l,
        }
    }
}

impl<T: EnumLike> IntoIterator for EnumVec<T> {
    type Item = T;
    type IntoIter = EnumVecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let l = self.len();
        EnumVecIntoIter {
            v: self,
            range: 0..l,
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
    range: Range<usize>,
}

impl<'a, T: EnumLike> Iterator for EnumVecIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|x| self.v.get(x).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    fn count(self) -> usize {
        self.size_hint().0
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth(n).map(|x| self.v.get(x).unwrap())
    }
}

impl<'a, T: EnumLike> DoubleEndedIterator for EnumVecIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().map(|x| self.v.get(x).unwrap())
    }
}

impl<'a, T: EnumLike> ExactSizeIterator for EnumVecIter<'a, T> {}

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
    range: Range<usize>,
}

impl<T: EnumLike> Iterator for EnumVecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|x| self.v.get(x).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    fn count(self) -> usize {
        self.size_hint().0
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth(n).map(|x| self.v.get(x).unwrap())
    }
}

impl<T: EnumLike> DoubleEndedIterator for EnumVecIntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().map(|x| self.v.get(x).unwrap())
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

impl<T: EnumLike> PartialEq for EnumVec<T> {
    fn eq(&self, other: &EnumVec<T>) -> bool {
        // TODO: efficient block-wise comparison
        if self.len() == other.len() {
            let l = self.len();
            for i in 0..l {
                unsafe { // Safe because we just checked the length
                    if self.get_raw_unchecked(i) != other.get_raw_unchecked(i) {
                        return false;
                    }
                }
            }

            true
        } else {
            false
        }
    }
}

impl<T: EnumLike> Eq for EnumVec<T> {}

impl<T: EnumLike> Hash for EnumVec<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let l = self.len();
        for i in 0..l {
            let x = unsafe { self.get_raw_unchecked(i) };
            x.hash(state);
        }
    }
}

// Warning: when implementing hash we must zero out the last block
// of the storage, otherwise the garbage data will make the hash inconsistent.
// Also, if we want to be generic over storage, we can't use the fast method
// of hashing each block, we must hash each element individually...

// Also, should we require T: Hash for impl Hash?

// Useful alias?
/// Alias for `EnumVec<bool>`
pub type BitVec = EnumVec<bool>;
// N-bit vec (currently unimplemented)
// needs const generics
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

    #[test]
    fn macro_enum_vec() {
        let a = vec![ABC::C, ABC::A, ABC::A, ABC::B, ABC::C];
        let b = enum_vec![ABC::C, ABC::A, ABC::A, ABC::B, ABC::C];

        assert_eq!(a, b.to_vec());

        let c = vec![ABC::C; 10];
        let d = enum_vec![ABC::C; 10];

        assert_eq!(c, d.to_vec());
    }

    #[test]
    fn clone_push_segfault() {
        let a: EnumVec<bool> = vec![false].into();
        let mut b = a.clone();
        // This push segfaults if the storage is malformed: for
        // example if we resize and set_raw_unchecked, but the storage
        // has len = 0. In that case the clone will not create a new storage,
        // and the push will segfault when trying to access an invalid pointer.
        b.push(true);
    }

    #[test]
    fn reserve_modifies_storage() {
        let mut ev = EnumVec::new();
        ev.reserve(1);
        assert_eq!(ev.len(), 0);
        assert!(ev.storage().len() > 0);
        ev.push(true);
        assert_eq!(ev.len(), 1);

        let mut a: EnumVec<bool> = EnumVec::new();
        unsafe { // This should be equivalent to push()
            a.reserve(1);
            // Assuming false == 0
            a.set_raw_unchecked(0, 0);
            a.set_len(1);
        }
        assert_eq!(a.len(), 1);
        assert_eq!(a.pop().unwrap(), false);
    }

    #[test]
    fn storage_resize() {
        let mut ev = EnumVec::new();
        unsafe {
            let s = ev.storage_mut();
            assert_eq!(s.len(), 0);
            s.resize(1, 0);
            assert_eq!(s.len(), 1);
        }
        ev.push(false);
    }
}
