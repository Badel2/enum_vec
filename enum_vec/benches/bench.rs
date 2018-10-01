// This file was initially taken from https://github.com/servo/rust-smallvec
// under the following LICENSE (MIT):
/*

Copyright (c) 2018 The Servo Project Developers

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.

*/

#![feature(test)]

#[macro_use]
extern crate enum_like_derive;
extern crate enum_like;
extern crate enum_vec;
extern crate test;

use self::test::Bencher;
use enum_vec::vec_u8::EnumVec as EnumVec8;
use enum_vec::vec_u16::EnumVec as EnumVec16;
use enum_vec::vec_u32::EnumVec as EnumVec32;
use enum_vec::vec_u64::EnumVec as EnumVec64;
use enum_vec::vec_u128::EnumVec as EnumVec128;
use enum_vec::smallvec_u32::EnumVec as SmallEnumVec32;
use enum_like::EnumLike;

const VEC_SIZE: usize = 16;
const SPILLED_SIZE: usize = 1000;

#[derive(Copy, Clone, Debug, EnumLike, PartialEq)]
struct T1(bool);
#[derive(Copy, Clone, Debug, EnumLike, PartialEq)]
struct T2(T1, T1);
#[derive(Copy, Clone, Debug, EnumLike, PartialEq)]
struct T4(T2, T2);
#[derive(Copy, Clone, Debug, EnumLike, PartialEq)]
struct T8(T4, T4);

impl From<u64> for T1 {
    fn from(n: u64) -> Self {
        Self::from_discr(n as usize % Self::NUM_VARIANTS)
    }
}
impl From<u64> for T2 {
    fn from(n: u64) -> Self {
        Self::from_discr(n as usize % Self::NUM_VARIANTS)
    }
}
impl From<u64> for T4 {
    fn from(n: u64) -> Self {
        Self::from_discr(n as usize % Self::NUM_VARIANTS)
    }
}
impl From<u64> for T8 {
    fn from(n: u64) -> Self {
        Self::from_discr(n as usize % Self::NUM_VARIANTS)
    }
}

trait Vector<T>: Extend<T> {
    fn new() -> Self;
    fn push(&mut self, val: T);
    fn pop(&mut self) -> Option<T>;
    fn remove(&mut self, p: usize) -> T;
    fn insert(&mut self, n: usize, val: T);
    fn from_elem(val: T, n: usize) -> Self;
    fn from_slice(val: &[T]) -> Self;
    fn any(&self, x: T) -> bool;
    fn all(&self, x: T) -> bool;
}

impl<T: Copy + EnumLike + PartialEq> Vector<T> for Vec<T> {
    fn new() -> Self {
        vec![]
    }

    fn push(&mut self, val: T) {
        self.push(val)
    }

    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    fn remove(&mut self, p: usize) -> T {
        self.remove(p)
    }

    fn insert(&mut self, n: usize, val: T) {
        self.insert(n, val)
    }

    fn from_elem(val: T, n: usize) -> Self {
        vec![val; n]
    }

    fn from_slice(val: &[T]) -> Self {
        val.into()
    }
    fn any(&self, x: T) -> bool {
        self.iter().any(|&a| a == x)
    }
    fn all(&self, x: T) -> bool {
        self.iter().all(|&a| a == x)
    }
}

macro_rules! impl_vector {
    ($( $typ:ty ),*) => {
        $(
            impl<T: Copy + EnumLike> Vector<T> for $typ {
                fn new() -> Self {
                    Self::new()
                }

                fn push(&mut self, val: T) {
                    self.push(val)
                }

                fn pop(&mut self) -> Option<T> {
                    self.pop()
                }

                fn remove(&mut self, p: usize) -> T {
                    self.remove(p)
                }

                fn insert(&mut self, n: usize, val: T) {
                    self.insert(n, val)
                }

                fn from_elem(val: T, n: usize) -> Self {
                    Self::from_elem(val, n)
                }

                fn from_slice(val: &[T]) -> Self {
                    Self::from_slice(val)
                }
                fn any(&self, x: T) -> bool {
                    self.any(x)
                }
                fn all(&self, x: T) -> bool {
                    self.all(x)
                }
            }
        )*
    }
}

impl_vector! {
    EnumVec8<T>, EnumVec16<T>, EnumVec32<T>, EnumVec64<T>, EnumVec128<T>,
    SmallEnumVec32<T>
}

macro_rules! make_benches {
    ($typ:ty { $($b_name:ident => $g_name:ident($($args:expr),*),)* }) => {
        $(
            #[bench]
            fn $b_name(b: &mut Bencher) {
                $g_name::<_, $typ>($($args,)* b)
            }
        )*
    }
}

macro_rules! make_make_benches {
    ($( $typ:ty => $mod:ident ),*) => {
        $(
pub mod $mod {
    use super::*;
    make_benches! {
        $typ {
            bench_push => gen_push(SPILLED_SIZE as _),
            bench_push_small => gen_push(VEC_SIZE as _),
            bench_insert => gen_insert(SPILLED_SIZE as _),
            bench_insert_small => gen_insert(VEC_SIZE as _),
            bench_insert_at_zero => gen_insert_at_zero(SPILLED_SIZE as _),
            bench_insert_at_zero_small => gen_insert_at_zero(VEC_SIZE as _),
            bench_remove => gen_remove(SPILLED_SIZE as _),
            bench_remove_small => gen_remove(VEC_SIZE as _),
            bench_remove_at_zero => gen_remove_at_zero(SPILLED_SIZE as _),
            bench_remove_at_zero_small => gen_remove_at_zero(VEC_SIZE as _),
            bench_extend => gen_extend(SPILLED_SIZE as _),
            bench_extend_small => gen_extend(VEC_SIZE as _),
            bench_from_slice => gen_from_slice(SPILLED_SIZE as _),
            bench_from_slice_small => gen_from_slice(VEC_SIZE as _),
            //bench_extend_from_slice => gen_extend_from_slice(SPILLED_SIZE as _),
            //bench_extend_from_slice_small => gen_extend_from_slice(VEC_SIZE as _),
            bench_macro_from_elem => gen_from_elem(SPILLED_SIZE as _),
            bench_macro_from_elem_small => gen_from_elem(VEC_SIZE as _),
            bench_pushpop => gen_pushpop(),
            bench_iter_all => iter_all(SPILLED_SIZE as _),
            bench_any => any_x(SPILLED_SIZE as _),
            bench_any_small => any_x(VEC_SIZE as _),
            bench_all => all_x(SPILLED_SIZE as _),
            bench_all_small => all_x(VEC_SIZE as _),
            bench_any_worst_case => any_worst_case(SPILLED_SIZE as _),
            bench_any_worst_case_small => any_worst_case(VEC_SIZE as _),
            bench_all_worst_case => all_worst_case(SPILLED_SIZE as _),
            bench_all_worst_case_small => all_worst_case(VEC_SIZE as _),
        }
    }

    fn iter_all<T: EnumLike + From<u64>, V: Vector<T>>(n: usize, b: &mut Bencher) {
        let v: $typ = (0..n as u64).map(|x| x.into()).collect();
        b.iter(|| {
            v.iter().fold(0, |sum, val| sum + val.to_discr())
        });
    }
}
            
        )*
    }
}

make_make_benches! {
    EnumVec8<T1> => enum_vec8_1,
    EnumVec16<T1> => enum_vec16_1,
    EnumVec32<T1> => enum_vec32_1,
    EnumVec64<T1> => enum_vec64_1,
    EnumVec128<T1> => enum_vec128_1,
    SmallEnumVec32<T1> => small_enum_vec32_1,
    Vec<T1> => normal_vec1,

    EnumVec8<T2> => enum_vec8_2,
    EnumVec16<T2> => enum_vec16_2,
    EnumVec32<T2> => enum_vec32_2,
    EnumVec64<T2> => enum_vec64_2,
    EnumVec128<T2> => enum_vec128_2,
    SmallEnumVec32<T2> => small_enum_vec32_2,
    Vec<T2> => normal_vec2,

    EnumVec8<T4> => enum_vec8_4,
    EnumVec16<T4> => enum_vec16_4,
    EnumVec32<T4> => enum_vec32_4,
    EnumVec64<T4> => enum_vec64_4,
    EnumVec128<T4> => enum_vec128_4,
    SmallEnumVec32<T4> => small_enum_vec32_4,
    Vec<T4> => normal_vec4,

    EnumVec8<T8> => enum_vec8_8,
    EnumVec16<T8> => enum_vec16_8,
    EnumVec32<T8> => enum_vec32_8,
    EnumVec64<T8> => enum_vec64_8,
    EnumVec128<T8> => enum_vec128_8,
    SmallEnumVec32<T8> => small_enum_vec32_8,
    Vec<T8> => normal_vec8
}

fn gen_push<T: EnumLike + From<u64>, V: Vector<T>>(n: u64, b: &mut Bencher) {
    #[inline(never)]
    fn push_noinline<T: EnumLike, V: Vector<T>>(vec: &mut V, x: T) {
        vec.push(x);
    }

    b.iter(|| {
        let mut vec = V::new();
        for x in 0..n {
            push_noinline(&mut vec, x.into());
        }
        vec
    });
}

fn gen_insert<T: EnumLike + From<u64>, V: Vector<T>>(n: u64, b: &mut Bencher) {
    #[inline(never)]
    fn insert_noinline<T: EnumLike + From<u64>, V: Vector<T>>(vec: &mut V, p: usize, x: T) {
        vec.insert(p, x)
    }

    b.iter(|| {
        let mut vec = V::new();
        // Add one element, with each iteration we insert one before the end.
        // This means that we benchmark the insertion operation and not the
        // time it takes to `ptr::copy` the data.
        vec.push(0.into());
        for x in 0..n {
            insert_noinline(&mut vec, x as _, x.into());
        }
        vec
    });
}

fn gen_insert_at_zero<T: EnumLike + From<u64>, V: Vector<T>>(n: u64, b: &mut Bencher) {
    #[inline(never)]
    fn insert_noinline<T: EnumLike + From<u64>, V: Vector<T>>(vec: &mut V, p: usize, x: T) {
        vec.insert(p, x)
    }

    b.iter(|| {
        let mut vec = V::new();
        // Add one element at the beginning, forcing to shift all
        // the data
        for x in 0..n {
            insert_noinline(&mut vec, 0, x.into());
        }
        vec
    });
}

fn gen_remove<T: EnumLike + From<u64>, V: Vector<T>>(n: usize, b: &mut Bencher) {
    #[inline(never)]
    fn remove_noinline<T: EnumLike + From<u64>, V: Vector<T>>(vec: &mut V, p: usize) -> T {
        vec.remove(p)
    }

    b.iter(|| {
        let mut vec = V::from_elem(2.into(), n as _);

        for x in (0..n - 1).rev() {
            remove_noinline(&mut vec, x);
        }
    });
}

fn gen_remove_at_zero<T: EnumLike + From<u64>, V: Vector<T>>(n: usize, b: &mut Bencher) {
    #[inline(never)]
    fn remove_noinline<T: EnumLike + From<u64>, V: Vector<T>>(vec: &mut V, p: usize) -> T {
        vec.remove(p)
    }

    b.iter(|| {
        let mut vec = V::from_elem(2.into(), n as _);

        for _ in (0..n - 1).rev() {
            remove_noinline(&mut vec, 0);
        }
    });
}

fn gen_extend<T: EnumLike + From<u64>, V: Vector<T>>(n: u64, b: &mut Bencher) {
    let v: Vec<T> = (0..n).map(|x| x.into()).collect();
    b.iter(|| {
        let mut vec = V::new();
        vec.extend(v.clone());
        vec
    });
}

fn gen_from_slice<T: EnumLike + From<u64>, V: Vector<T>>(n: u64, b: &mut Bencher) {
    let v: Vec<T> = (0..n).map(|x| x.into()).collect();
    b.iter(|| {
        let vec = V::from_slice(&v);
        vec
    });
}

fn gen_pushpop<T: EnumLike + From<u64>, V: Vector<T>>(b: &mut Bencher) {
    #[inline(never)]
    fn pushpop_noinline<T: EnumLike + From<u64>, V: Vector<T>>(vec: &mut V, x: T) -> Option<T> {
        vec.push(x);
        vec.pop()
    }

    b.iter(|| {
        let mut vec = V::new();
        for x in 0..SPILLED_SIZE as _ {
            pushpop_noinline(&mut vec, x.into());
        }
        vec
    });
}

fn gen_from_elem<T: EnumLike + From<u64>, V: Vector<T>>(n: usize, b: &mut Bencher) {
    b.iter(|| {
        let vec = V::from_elem(2.into(), n);
        vec
    });
}

fn any_x<T: EnumLike + From<u64>, V: Vector<T>>(n: u64, b: &mut Bencher) {
    let vall = V::from_elem(2.into(), n as usize);
    let asdf: Vec<_> = (0..n).map(|x| x.into()).collect();
    let vany = V::from_slice(&asdf);

    b.iter(|| {
        let mut count = 0;
        let x = 2;
        count += vall.any(x.into()) as i32;
        count += vany.any(x.into()) as i32;
        count
    });
}

fn any_worst_case<T: EnumLike + From<u64>, V: Vector<T>>(n: u64, b: &mut Bencher) {
    let vall = V::from_elem(2.into(), n as usize);

    b.iter(|| {
        let mut count = 0;
        let x = 1;
        count += vall.any(x.into()) as i32;
        assert_eq!(count, 0);
        count
    });
}

fn all_x<T: EnumLike + From<u64>, V: Vector<T>>(n: u64, b: &mut Bencher) {
    let vall = V::from_elem(2.into(), n as usize);
    let asdf: Vec<_> = (0..n).map(|x| x.into()).collect();
    let vany = V::from_slice(&asdf);

    b.iter(|| {
        let mut count = 0;
        let x = 2;
        count += vall.all(x.into()) as i32;
        count += vany.all(x.into()) as i32;
        count
    });
}

fn all_worst_case<T: EnumLike + From<u64>, V: Vector<T>>(n: u64, b: &mut Bencher) {
    let vall = V::from_elem(2.into(), n as usize);

    b.iter(|| {
        let mut count = 0;
        let x = 2;
        count += vall.all(x.into()) as i32;
        assert_eq!(count, 1);
        count
    });
}

/*
#[bench]
fn bench_insert_many(b: &mut Bencher) {
    #[inline(never)]
    fn insert_many_noinline<I: IntoIterator<Item = u64>>(
        vec: &mut EnumVec<T2>,
        index: usize,
        iterable: I,
    ) {
        vec.insert_many(index, iterable)
    }

    b.iter(|| {
        let mut vec = SmallVec::<[u64; VEC_SIZE]>::new();
        insert_many_noinline(&mut vec, 0, 0..SPILLED_SIZE as _);
        insert_many_noinline(&mut vec, 0, 0..SPILLED_SIZE as _);
        vec
    });
}

#[bench]
fn bench_insert_from_slice(b: &mut Bencher) {
    let v: Vec<u64> = (0..SPILLED_SIZE as _).collect();
    b.iter(|| {
        let mut vec = EnumVec::new();
        vec.insert_from_slice(0, &v);
        vec.insert_from_slice(0, &v);
        vec
    });
}
*/
