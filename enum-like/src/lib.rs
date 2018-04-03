#![deny(missing_docs)]
// TODO: abstract over bit blocks
// or maybe abstract over vec? So we can use SmallVec...
// If the enum has more than 128 variants, just use Vec<Enum>
// Should we require T to be Clone?

//! This crate provides the `EnumLike` trait, which defines a mapping from
//! a given type to `usize`.
//!
//! This is similar to `std::mem::discriminant`, however it has a few
//! differences. First of all, all the values are consecutive starting from
//! zero. This means that if an enum has 10 variants, the discriminant will
//! always be lower than 10. If a field has an explicit discriminant, that
//! value is ignored: in
//! `enum { A = 100 }`, `A` will have the value of 0.
//! And most importantly, this trait allows to create
//! an instance of the type from the `usize`, because the enum data, if present,
//! is also encoded in the discriminant (if possible). For example:
//!
//! ```norun
//! enum ABC { A, B, C }
//! enum DEF { D, E, F }
//! enum AD { A(ABC), D(DEF) }
//! ```
//!
//! The `AD` enum has 2 variants, but since each of these variants is an enum
//! with 3 variants, the `AD::values().count()` will return 6 instead of 2.

/// The `EnumLike` trait specifies how a type will be stored inside the
/// `EnumVec`.
///
/// It associates every possible instance of the type with a number. However
/// this number does not need to be the same as the result of a simple
/// `enum as usize` cast.
///
/// This trait is unsafe because implementations must follow the contract,
/// especially the first rule:
/// * `self.to_discr()` returns a value `x < NUM_VARIANTS`
/// * `Self::from_discr(self.to_discr()) == self`
/// * `Self::from_discr(x)` is only required to handle valid values of `x`
///
/// # Example 1
///
/// ```
/// use enum_like::EnumLike;
///
/// enum ExtendedBool {
///     True,
///     False,
///     FileNotFound,
/// }
///
/// unsafe impl EnumLike for ExtendedBool {
///     const NUM_VARIANTS: usize = 3;
///
///     fn to_discr(self) -> usize {
///         match self {
///             ExtendedBool::True => 0,
///             ExtendedBool::False => 1,
///             ExtendedBool::FileNotFound => 2,
///         }
///     }
///
///     fn from_discr(x: usize) -> Self {
///         match x {
///             0 => ExtendedBool::True,
///             1 => ExtendedBool::False,
///             2 => ExtendedBool::FileNotFound,
///             _ => unreachable!(),
///         }
///     }
/// }
/// ```
///
/// # Example 2
///
/// ```
/// use enum_like::EnumLike;
///
/// enum SomeFlags {
///     Read = 4,
///     Write = 2,
///     Exec = 1,
/// }
///
/// unsafe impl EnumLike for SomeFlags {
///     const NUM_VARIANTS: usize = 3;
///
///     fn to_discr(self) -> usize {
///         match self {
///             // We override the default values, because 4 is out of range,
///             // but we could also increase NUM_VARIANTS to 5 instead.
///             SomeFlags::Read => 0,
///             SomeFlags::Write => 1,
///             SomeFlags::Exec => 2,
///
///         }
///     }
///
///     fn from_discr(x: usize) -> Self {
///         match x {
///             0 => SomeFlags::Read,
///             1 => SomeFlags::Write,
///             2 => SomeFlags::Exec,
///             _ => unreachable!(),
///         }
///     }
/// }
/// ```
///
/// # Example 3
/// Of course, it is not limited to enums:
///
/// ```
/// use enum_like::EnumLike;
///
/// struct Digit {
///     x: u8, // x >= 0 && x <= 9
/// }
///
/// unsafe impl EnumLike for Digit {
///     const NUM_VARIANTS: usize = 10;
///     fn to_discr(self) -> usize {
///         self.x as usize
///     }
///     fn from_discr(x: usize) -> Self {
///         let x = x as u8;
///         Self { x }
///     }
/// }
/// ```
///
/// Here it is important to make sure that the `Digit` will always have a valid
/// value in the [0, 9] range. Otherwise, if `self.to_discr()` returns any number
/// bigger than `NUM_VARIANTS`, everything breaks.
///
pub unsafe trait EnumLike {
    /// The number of variants of this type
    const NUM_VARIANTS: usize;

    /// Convert type to discriminant
    fn to_discr(self) -> usize;

    /// Get type instance from discriminant
    // We could have a static array with all the instances of
    // the type and just return INSTANCE[x], we could maybe do that with
    // lazy_static but I hope the compiler is smart enough to optimize the
    // match {} into something similar
    fn from_discr(x: usize) -> Self;
}

// TODO: impl ! with NUM_VARIANTS = 0

// Any one-variant types can be trivially implemented
unsafe impl EnumLike for () {
    const NUM_VARIANTS: usize = 1;
    fn to_discr(self) -> usize {
        0
    }
    fn from_discr(_x: usize) -> Self {
        ()
    }
}

unsafe impl EnumLike for bool {
    const NUM_VARIANTS: usize = 2;
    fn to_discr(self) -> usize {
        //self as usize
        // Just to be sure
        if self {
            1
        } else {
            0
        }
    }
    fn from_discr(x: usize) -> Self {
        x != 0
    }
}

// This is different from the usual optimization, where Option<bool> is
// 0: Some(false), 1: Some(true), 2: None
// This implementation is:
// 0: None, 1: Some(false), 2: Some(true)
unsafe impl<T: EnumLike> EnumLike for Option<T> {
    const NUM_VARIANTS: usize = 1 + T::NUM_VARIANTS;
    fn to_discr(self) -> usize {
        match self {
            None => 0,
            Some(x) => 1 + x.to_discr(),
        }
    }
    fn from_discr(x: usize) -> Self {
        match x {
            0 => None,
            x => Some(T::from_discr(x - 1)),
        }
    }
}

unsafe impl<T: EnumLike, S: EnumLike> EnumLike for Result<T, S> {
    const NUM_VARIANTS: usize = T::NUM_VARIANTS + S::NUM_VARIANTS;
    fn to_discr(self) -> usize {
        match self {
            Ok(x) => x.to_discr(),
            Err(x) => T::NUM_VARIANTS + x.to_discr(),
        }
    }
    fn from_discr(x: usize) -> Self {
        match x {
            x if x < T::NUM_VARIANTS => Ok(T::from_discr(x)),
            x => Err(S::from_discr(x - T::NUM_VARIANTS)),
        }
    }
}

// TODO: macro for impl more tuple types?
unsafe impl<T: EnumLike> EnumLike for (T,) {
    const NUM_VARIANTS: usize = T::NUM_VARIANTS;
    fn to_discr(self) -> usize {
        self.0.to_discr()
    }
    fn from_discr(x: usize) -> Self {
        (T::from_discr(x),)
    }
}

unsafe impl<T: EnumLike, S: EnumLike> EnumLike for (T, S) {
    const NUM_VARIANTS: usize = T::NUM_VARIANTS * S::NUM_VARIANTS;
    fn to_discr(self) -> usize {
        self.0.to_discr() + self.1.to_discr() * T::NUM_VARIANTS
    }
    fn from_discr(x: usize) -> Self {
        //(T::from_discr(x % T::NUM_VARIANTS), S::from_discr(x / T::NUM_VARIANTS))
        // workarround for #45850
        (
            T::from_discr(x.wrapping_rem(T::NUM_VARIANTS)),
            S::from_discr(x.wrapping_div(T::NUM_VARIANTS)),
        )
    }
}

// (A, B, C) == ((A, B), C)
// generic implementation needs
// https://github.com/rust-lang/rfcs/pull/1935
/*
unsafe impl<A: EnumLike, B: EnumLike, C: EnumLike> EnumLike for (A, B, C) {
    const NUM_VARIANTS: usize = <((A, B), C)>::NUM_VARIANTS;
    fn to_discr(self) -> usize {
        ((self.0, self.1), self.2).to_discr()
    }
    fn from_discr(x: usize) -> Self {
        let a = <((A, B), C)>::from_discr(x);
        ((a.0).0, (a.0).1, a.1)
    }
}

unsafe impl<A: EnumLike, B: EnumLike, C: EnumLike, D: EnumLike> EnumLike for (A, B, C, D) {
    const NUM_VARIANTS: usize = <((A, B, C), D)>::NUM_VARIANTS;
    fn to_discr(self) -> usize {
        ((self.0, self.1, self.2), self.3).to_discr()
    }
    fn from_discr(x: usize) -> Self {
        let a = <((A, B, C), D)>::from_discr(x);
        ((a.0).0, (a.0).1, (a.0).2, a.1)
    }
}
*/

// macro for implementing n-ary tuple functions and operations
macro_rules! tuple_impls {
    (   $Tuple:ident {
            ($last_idx:tt) -> $last_T:ident
            $(($idx:tt) -> $T:ident)+
        }
    ) => {
unsafe impl<$($T: EnumLike,)+ $last_T: EnumLike> EnumLike for ($($T,)+ $last_T) {
    const NUM_VARIANTS: usize = <(($($T,)+), $last_T)>::NUM_VARIANTS;
    fn to_discr(self) -> usize {
        (reverse_idx_b!(self [$($idx)+]), self.$last_idx).to_discr()
    }
    fn from_discr(x: usize) -> Self {
        //let a = <(reverse!([$($T)+]), $last_T)>::from_discr(x);
        let a = <(($($T,)+), $last_T)>::from_discr(x);
        //((a.0).0, (a.0).1, a.1)
        reverse_idx_a!(a [$($idx)+])
    }
}
    }
}

macro_rules! reverse_idx_b {
    ($a:ident [] $($reversed:tt)*) => {
        ($($a.$reversed,)+)  // base case
    };
    ($a:ident [$first:tt $($rest:tt)*] $($reversed:tt)*) => {
        reverse_idx_b!($a [$($rest)*] $first $($reversed)*)  // recursion
    };
}
macro_rules! reverse_idx_a {
    ($a:ident [] $($reversed:tt)*) => {
        ($(($a.0).$reversed,)+ $a.1)  // base case
    };
    ($a:ident [$first:tt $($rest:tt)*] $($reversed:tt)*) => {
        reverse_idx_a!($a [$($rest)*] $first $($reversed)*)  // recursion
    };
}

macro_rules! recursive_tuple_impls {
    {
        ($idx0:tt) -> $T0:ident
        ($idx1:tt) -> $T1:ident
    } => {
        // Done, impls for (A, B) are hardcoded
    };
    {
        ($last_idx:tt) -> $last_T:ident
        $(($idx:tt) -> $T:ident)+
    } => {
        tuple_impls! {
            TupleX {
                ($last_idx) -> $last_T
                $(($idx) -> $T)+
            }
        }

        recursive_tuple_impls! {
            $(($idx) -> $T)+
        }
    };
}

recursive_tuple_impls! {
    (31) -> A31
    (30) -> A30
    (29) -> A29
    (28) -> A28
    (27) -> A27
    (26) -> A26
    (25) -> A25
    (24) -> A24
    (23) -> A23
    (22) -> A22
    (21) -> A21
    (20) -> A20
    (19) -> A19
    (18) -> A18
    (17) -> A17
    (16) -> A16
    (15) -> A15
    (14) -> A14
    (13) -> A13
    (12) -> A12
    (11) -> A11
    (10) -> A10
    (9) -> A9
    (8) -> A8
    (7) -> A7
    (6) -> A6
    (5) -> A5
    (4) -> A4
    (3) -> A3
    (2) -> A2
    (1) -> A1
    (0) -> A0
}

// TODO: array impls with macro
// we can just transform [T; 2] into (T, T)
// and [T; 0] into ()
unsafe impl<T: EnumLike> EnumLike for [T; 0] {
    const NUM_VARIANTS: usize = <()>::NUM_VARIANTS;
    fn to_discr(self) -> usize {
        0
    }
    fn from_discr(_x: usize) -> Self {
        []
    }
}
unsafe impl<T: EnumLike> EnumLike for [T; 1] {
    const NUM_VARIANTS: usize = T::NUM_VARIANTS;
    fn to_discr(mut self) -> usize {
        use std::mem;
        let x0 = unsafe { mem::replace(&mut self[0], mem::uninitialized()) };
        x0.to_discr()
    }
    fn from_discr(x: usize) -> Self {
        [T::from_discr(x)]
    }
}
unsafe impl<T: EnumLike> EnumLike for [T; 2] {
    const NUM_VARIANTS: usize = <(T, T)>::NUM_VARIANTS;
    fn to_discr(mut self) -> usize {
        use std::mem;
        let x0 = unsafe { mem::replace(&mut self[0], mem::uninitialized()) };
        let x1 = unsafe { mem::replace(&mut self[1], mem::uninitialized()) };
        (x0, x1).to_discr()
    }
    fn from_discr(x: usize) -> Self {
        let t = <(T, T)>::from_discr(x);
        [t.0, t.1]
    }
}
unsafe impl<T: EnumLike> EnumLike for [T; 3] {
    const NUM_VARIANTS: usize = <(T, T, T)>::NUM_VARIANTS;
    fn to_discr(mut self) -> usize {
        use std::mem;
        let x0 = unsafe { mem::replace(&mut self[0], mem::uninitialized()) };
        let x1 = unsafe { mem::replace(&mut self[1], mem::uninitialized()) };
        let x2 = unsafe { mem::replace(&mut self[2], mem::uninitialized()) };
        (x0, x1, x2).to_discr()
    }
    fn from_discr(x: usize) -> Self {
        let t = <(T, T, T)>::from_discr(x);
        [t.0, t.1, t.2]
    }
}

/// Helper trait to iterate over all the possible values of an enum.
/// Note: you don't need to implement this trait, it is provided by `EnumLike`.
///
/// Common usage: `for i in T::values() {}`
/// # Example 1
///
/// You first need to `impl EnumLike` to get access to this trait.
///
/// ```
/// use enum_like::{EnumLike, EnumValues};
///
/// #[derive(Debug)]
/// enum ABC { A, B, C }
///
/// unsafe impl EnumLike for ABC {
///     const NUM_VARIANTS: usize = 3;
///     fn to_discr(self) -> usize {
///         match self {
///             ABC::A => 0,
///             ABC::B => 1,
///             ABC::C => 2,
///         }
///     }
///     fn from_discr(x: usize) -> Self {
///         match x {
///             0 => ABC::A,
///             1 => ABC::B,
///             2 => ABC::C,
///             _ => unreachable!(),
///         }
///     }
/// }
///
/// fn main() {
///     for i in ABC::values() {
///         println!("{:?}", i);
///     }
/// }
/// ```
///
/// Output:
/// ```norun
/// A
/// B
/// C
/// ```
///
/// # Example 2
/// The `EnumLike` trait is implemented by default for `bool` and `Option` types,
/// so you can do stuff like:
///
/// ```
/// use enum_like::EnumValues;
/// type NestedOptionBool = Option<Option<Option<bool>>>;
///
/// fn main() {
///     for (idx, i) in NestedOptionBool::values().enumerate() {
///         println!("{}: {:?}", idx, i);
///     }
/// }
/// ```
///
/// Output:
/// ```norun
/// 0: None
/// 1: Some(None)
/// 2: Some(Some(None))
/// 3: Some(Some(Some(false)))
/// 4: Some(Some(Some(true)))
/// ```
///
/// The index can then be used to create an instance of the type using
/// `NestedOptionBool::from_discr(x)`
pub trait EnumValues: EnumLike {
    /// Returns an iterator over the values of `Self`
    fn values() -> Values<Self>
    where
        Self: Sized,
    {
        Values::of()
    }
}

impl<T: EnumLike> EnumValues for T {}

use std::marker::PhantomData;

/// Iterator over the values (variants) of `T`
///
/// See [EnumValues](trait.EnumValues.html) for more information
#[derive(Copy, Clone, Debug)]
pub struct Values<T: EnumLike> {
    current: usize,
    max: usize,
    _p: PhantomData<T>,
}

impl<T: EnumLike> Values<T> {
    fn of() -> Self {
        Self {
            current: 0,
            max: T::NUM_VARIANTS,
            _p: PhantomData,
        }
    }
}

impl<T: EnumLike> Iterator for Values<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.max {
            let x = T::from_discr(self.current);
            self.current += 1;

            Some(x)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_elements = (self.max - self.current) as usize;

        (remaining_elements, Some(remaining_elements))
    }

    fn count(self) -> usize {
        let remaining_elements = (self.max - self.current) as usize;

        remaining_elements
    }

    fn last(mut self) -> Option<Self::Item> {
        if self.max == 0 || self.current == self.max {
            None
        } else {
            self.current = self.max - 1;

            self.next()
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.current + n < self.max {
            self.current += n;

            self.next()
        } else {
            self.current = self.max;

            None
        }
    }
}

impl<T: EnumLike> ExactSizeIterator for Values<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    fn check_values_of<T: Clone + ::std::fmt::Debug + PartialEq + EnumLike>(
        x: usize,
    ) {
        let mut seen = vec![];
        let mut counter = 0;
        for i in T::values() {
            seen.push(i.clone());
            let idx = i.clone().to_discr();
            assert_eq!(i, T::from_discr(idx));
            assert_eq!(idx, T::from_discr(idx).to_discr());
            counter += 1;
        }

        assert_eq!(counter, T::NUM_VARIANTS);

        // check that each element in seen only appears once
        // in o(n^2) time because T is not Hash or Ord or anything
        for i in 0..counter {
            for j in i + 1..counter {
                if seen[i] == seen[j] {
                    panic!("Duplicate entry for {:?}", seen[i]);
                }
            }
        }

        assert_eq!(x, T::NUM_VARIANTS);
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum ABC {
        A,
        B,
        C,
    }

    unsafe impl EnumLike for ABC {
        const NUM_VARIANTS: usize = 3;
        fn to_discr(self) -> usize {
            //self as u8
            // ^this may not work if the enum has variants with values, like A = 100:
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
    fn values_of() {
        let mut a = ABC::values();
        assert_eq!(a.next(), Some(ABC::A));
        assert_eq!(a.next(), Some(ABC::B));
        assert_eq!(a.next(), Some(ABC::C));
        assert_eq!(a.next(), None);

        let b = Digit::values();
        assert_eq!(b.count(), 10);

        let c = TwoDigits::values();
        assert_eq!(c.count(), 100);
        //panic!("Success!");
    }

    #[test]
    fn check_builtin_impls() {
        check_values_of::<()>(1);
        check_values_of::<bool>(2);
        check_values_of::<Option<()>>(2);
        check_values_of::<Option<bool>>(3);
        check_values_of::<Result<(), ()>>(2);
        check_values_of::<Result<bool, ()>>(3);
        check_values_of::<Result<(), bool>>(3);
        check_values_of::<Result<bool, bool>>(4);

        check_values_of::<((),)>(1);
        check_values_of::<((), ())>(1);
        check_values_of::<((), (), ())>(1);
        check_values_of::<(bool,)>(2);
        check_values_of::<(bool, bool)>(2 * 2);
        check_values_of::<(bool, bool, bool)>(2 * 2 * 2);
    }

    #[test]
    fn check_array_impls() {
        // Helper function to convert from u8 to [bool; N]
        // used to test everything
        fn as_bool_vec(mut x: u8, n: usize) -> Vec<bool> {
            let mut v = vec![];
            while x != 0 {
                v.push((x & 1) == 1);
                x = x >> 1;
            }

            while v.len() < n {
                v.push(false);
            }

            v
        }

        macro_rules! test_array_impl_n {
            ( $( $n:expr ),* ) => {
                $(
                    let mut a = <[bool; $n]>::values();
                    for i in 0..(1 << $n) {
                        assert_eq!(a.next().unwrap(), *as_bool_vec(i, $n));
                    }
                    assert_eq!(a.next(), None);
                )*
            };
        }

        test_array_impl_n!(0, 1, 2, 3);

    }

    #[test]
    fn check_tuple_impls() {
        let a = <(bool, bool, bool, bool)>::values();
        for i in a {
            println!("{:?}", i);
            println!("{:?}", i.to_discr());
        }
        //panic!("Done!");
    }

    #[test]
    fn check_test_impls() {
        check_values_of::<ABC>(3);
        check_values_of::<Digit>(10);
        check_values_of::<TwoDigits>(100);
    }

    #[test]
    fn nested_option_bool() {
        type NestedOptionBool = Option<Option<Option<bool>>>;
        check_values_of::<NestedOptionBool>(5);
        let mut a = NestedOptionBool::values();

        // The ordering is important, if it changes remember to update the doc
        // example
        assert_eq!(a.next().unwrap(), None);
        assert_eq!(a.next().unwrap(), Some(None));
        assert_eq!(a.next().unwrap(), Some(Some(None)));
        assert_eq!(a.next().unwrap(), Some(Some(Some(false))));
        assert_eq!(a.next().unwrap(), Some(Some(Some(true))));
        assert_eq!(a.next(), None);
    }

    #[test]
    fn result_option_unit() {
        type Abomination =
            Result<Result<Option<()>, bool>, Result<(), Option<bool>>>;

        check_values_of::<Abomination>(8);
        assert_eq!(
            Abomination::values().nth(1).unwrap(),
            Ok(Ok(Some(())))
        );
        assert_eq!(
            Abomination::values().last().unwrap(),
            Err(Err(Some(true)))
        );
    }
}
