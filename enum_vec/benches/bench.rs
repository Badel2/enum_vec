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
use enum_vec::EnumVec;
use enum_like::EnumLike;

const VEC_SIZE: usize = 16;
const SPILLED_SIZE: usize = 1000;

#[derive(Copy, Clone, Debug, EnumLike)]
enum T2 { A, B, C, D }

impl From<u64> for T2 {
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
}

impl<T: Copy + EnumLike> Vector<T> for Vec<T> {
    fn new() -> Self {
        Self::with_capacity(VEC_SIZE)
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
}

impl<T: Copy + EnumLike> Vector<T> for EnumVec<T> {
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
        let mut ev = EnumVec::new();
        ev.resize(n, val);

        ev
    }
}

macro_rules! make_benches {
    ($typ:ty { $($b_name:ident => $g_name:ident($($args:expr),*),)* }) => {
        $(
            #[bench]
            fn $b_name(b: &mut Bencher) {
                $g_name::<$typ>($($args,)* b)
            }
        )*
    }
}


pub mod enum_vec2 {
    use super::*;
    make_benches! {
        EnumVec<T2> {
            bench_push => gen_push(SPILLED_SIZE as _),
            bench_push_small => gen_push(VEC_SIZE as _),
            bench_insert => gen_insert(SPILLED_SIZE as _),
            bench_insert_small => gen_insert(VEC_SIZE as _),
            bench_remove => gen_remove(SPILLED_SIZE as _),
            bench_remove_small => gen_remove(VEC_SIZE as _),
            bench_extend => gen_extend(SPILLED_SIZE as _),
            bench_extend_small => gen_extend(VEC_SIZE as _),
            //bench_from_slice => gen_from_slice(SPILLED_SIZE as _),
            //bench_from_slice_small => gen_from_slice(VEC_SIZE as _),
            //bench_extend_from_slice => gen_extend_from_slice(SPILLED_SIZE as _),
            //bench_extend_from_slice_small => gen_extend_from_slice(VEC_SIZE as _),
            bench_macro_from_elem => gen_from_elem(SPILLED_SIZE as _),
            bench_macro_from_elem_small => gen_from_elem(VEC_SIZE as _),
            bench_pushpop => gen_pushpop(),
        }
    }
}

pub mod normal_vec2 {
    use super::*;
    make_benches! {
        Vec<T2> {
            bench_push => gen_push(SPILLED_SIZE as _),
            bench_push_small => gen_push(VEC_SIZE as _),
            bench_insert => gen_insert(SPILLED_SIZE as _),
            bench_insert_small => gen_insert(VEC_SIZE as _),
            bench_remove => gen_remove(SPILLED_SIZE as _),
            bench_remove_small => gen_remove(VEC_SIZE as _),
            bench_extend => gen_extend(SPILLED_SIZE as _),
            bench_extend_small => gen_extend(VEC_SIZE as _),
            //bench_from_slice_vec => gen_from_slice(SPILLED_SIZE as _),
            //bench_from_slice_vec_small => gen_from_slice(VEC_SIZE as _),
            //bench_extend_from_slice_vec => gen_extend_from_slice(SPILLED_SIZE as _),
            //bench_extend_from_slice_vec_small => gen_extend_from_slice(VEC_SIZE as _),
            bench_macro_from_elem => gen_from_elem(SPILLED_SIZE as _),
            bench_macro_from_elem_small => gen_from_elem(VEC_SIZE as _),
            bench_pushpop => gen_pushpop(),
        }
    }
}

fn gen_push<V: Vector<T2>>(n: u64, b: &mut Bencher) {
    #[inline(never)]
    fn push_noinline<V: Vector<T2>>(vec: &mut V, x: T2) {
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

fn gen_insert<V: Vector<T2>>(n: u64, b: &mut Bencher) {
    #[inline(never)]
    fn insert_noinline<V: Vector<T2>>(vec: &mut V, p: usize, x: T2) {
        vec.insert(p, x)
    }

    b.iter(|| {
        let mut vec = V::new();
        // Add one element, with each iteration we insert one before the end.
        // This means that we benchmark the insertion operation and not the
        // time it takes to `ptr::copy` the data.
        vec.push(T2::A);
        for x in 0..n {
            insert_noinline(&mut vec, x as _, x.into());
        }
        vec
    });
}

fn gen_remove<V: Vector<T2>>(n: usize, b: &mut Bencher) {
    #[inline(never)]
    fn remove_noinline<V: Vector<T2>>(vec: &mut V, p: usize) -> T2 {
        vec.remove(p)
    }

    b.iter(|| {
        let mut vec = V::from_elem(T2::C, n as _);

        for x in (0..n - 1).rev() {
            remove_noinline(&mut vec, x);
        }
    });
}

fn gen_extend<V: Vector<T2>>(n: u64, b: &mut Bencher) {
    let v: Vec<T2> = (0..n).map(|x| x.into()).collect();
    b.iter(|| {
        let mut vec = V::new();
        vec.extend(v.clone());
        vec
    });
}

/*
fn gen_from_slice<V: Vector<T2>>(n: u64, b: &mut Bencher) {
    let v: Vec<T2> = (0..n).map(|x| x.into()).collect();
    b.iter(|| {
        let vec = V::from(&v);
        vec
    });
}
*/

fn gen_pushpop<V: Vector<T2>>(b: &mut Bencher) {
    #[inline(never)]
    fn pushpop_noinline<V: Vector<T2>>(vec: &mut V, x: T2) -> Option<T2> {
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

fn gen_from_elem<V: Vector<T2>>(n: usize, b: &mut Bencher) {
    b.iter(|| {
        let vec = V::from_elem(T2::C, n);
        vec
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