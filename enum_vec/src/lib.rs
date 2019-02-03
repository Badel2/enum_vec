#![deny(missing_docs)]

//! A vector which efficiently stores enum variants.

extern crate enum_like;
#[cfg(feature = "smallvec")]
extern crate smallvec;

/// Not sure if this is needed
pub use enum_like::*;

// Idea for SmallEnumVec: literally copy paste the code
// s/Vec/SmallVec
// Or maybe
// storage: union { Vec<u32>, [usize; 3] }
// Tag: MSB of num_elements

/// Macro for easy initialization similar to `vec!`:
///
/// ```
/// extern crate enum_vec;
/// use enum_vec::EnumVec;
/// use enum_vec::enum_vec;
///
/// let ev1 = enum_vec![true, true, false];
/// let ev2 = enum_vec![true; 16];
/// ```
#[macro_export]
macro_rules! enum_vec {
    ($elem:expr; $n:expr) => ({
        EnumVec::from_elem($elem, $n)
    });
    ($($x:expr),*$(,)*) => ({
        EnumVec::from_slice(&[$($x),*])
    });
}

/// Alternative implementation of `EnumVec` with `Vec<u8>` storage.
pub mod vec_u8;
/// Alternative implementation of `EnumVec` with `Vec<u16>` storage.
pub mod vec_u16;
/// Default `EnumVec` with `Vec<u32>` storage.
pub mod vec_u32;
pub use vec_u32::EnumVec;
/// Alternative implementation of `EnumVec` with `Vec<u64>` storage.
pub mod vec_u64;
/// Alternative implementation of `EnumVec` with `Vec<u128>` storage.
pub mod vec_u128;

#[cfg(feature = "smallvec")]
/// `SmallEnumVec`
pub mod smallvec_u32;
