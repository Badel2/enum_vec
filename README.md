# enum-vec
Efficiently store a vector of enum variants

[![Crates.io](https://img.shields.io/crates/v/enum_vec.svg)](https://crates.io/crates/enum_vec)
![Crates.io](https://img.shields.io/crates/l/enum_vec.svg)
[![Build Status](https://travis-ci.org/Badel2/enum_vec.svg?branch=master)](https://travis-ci.org/Badel2/enum_vec)

[Documentation](https://badel2.github.io/enum_vec/enum_vec/index.html)

Let's say you have an enum `Direction` with 4 variants. You only need 2 bits to
store the discriminant, but Rust will use the minimum of 1 byte (8 bits).
Therefore, when using a `Vec<Direction>` with 16 elements it will use 16 bytes
of memory. However, this crate provides the `EnumVec` type, which only uses as
many bits as needed. So a `EnumVec<Direction>` with 16 elements will only use
4 bytes of memory.

# Implementation
Since Rust doesn't provide a way to count the variants of a type, the
`enum_like` crate defines a trait `EnumLike` with an associated constant
`NUM_VARIANTS`, and helper methods to convert from `usize` to `T`. This trait
is implemented for a few common types, like `bool` and `Option<T>`, and can be
implemented for any type. The implementation can be automated using the
`enum_like_derive` crate, which provides the `#[derive(EnumLike)]` proc macro.

# Example
Add this to your `Cargo.toml`:
```
[dependencies]
enum_vec = "0.3"
enum_like = "0.2"
enum_like_derive = "0.1"
```

And then in `src/main.rs`:
```rust
#[macro_use]
extern crate enum_like_derive;
extern crate enum_like;
extern crate enum_vec;

use enum_vec::EnumVec;

#[derive(Copy, Clone, Debug, EnumLike)]
enum Direction {
    Left, Right, Up, Down,
}

fn main() {
    let mut v = EnumVec::new();
    v.push(Direction::Left);
    v.push(Direction::Right);
    v.push(Direction::Left);
    v.push(Direction::Right);

    for d in v {
        println!("{:?}", d);
    }
}
```

See `examples/src/main.rs` for more usage examples.

# BitVec
Since an EnumVec is essentially a n-bitvec, you can use it as such.
```rust
type BitVec = EnumVec<bool>;
type TwoBitVec = EnumVec<[bool; 2]>;
type TwoBitVec = EnumVec<(bool, bool)>;
type FourBitVec = EnumVec<[bool; 4]>;
```

# Deriving EnumLike
You can automatically derive `EnumLike` for almost any type, as long as all of
its fields are `EnumLike`.

```rust
struct BitField {
    opt_0: bool,
    opt_1: bool,
    opt_2: bool,
    opt_3: bool,
}
enum BitsOrRaw {
    Bits(BitField),
    Raw { opt_01: (bool, bool), opt_23: (bool, bool), },
}
```

# impl EnumLike

You can write a custom `EnumLike` implementation: the following code allows
to create a `EnumVec<Digit>` where each element is 4 bits, instead of the 8
required by `u8`.

```rust
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
```

This trait is unsafe because other code assumes that `to_discr()` never returns
something `>=` than `NUM_VARIANTS`.

# Memory efficiency
Since by default each block is 32 bits, the `EnumVec` is only 100% memory
efficient when each element is 1, 2, 4, 8, 16 or 32 bits long.
That's because the elements are never split across two blocks: a 15-bit element
stored inside a 32-bit block will always use 30 bits and waste the remaining 2.
In general, the efficiency
can be calculated as `1 - (32 % n) / 32`, but it's always
equal or better than a normal `Vec`. However it's equal when n >= 11, so if
you have a type with 2048 variants, you should consider using a `Vec` instead.

| n   | Vec | EnumVec8 | EnumVec16 | EnumVec32 | EnumVec64 | EnumVec128 |
| --- | -------- | ----- | ------ | ------- | -------- | --------- |
| 1   | 0.125    | 1     | 1      | 1       | 1        | 1         |
| 2   | 0.25     | 1     | 1      | 1       | 1        | 1         |
| 3   | 0.375    | 0.75  | 0.9375 | 0.9375  | 0.984375 | 0.984375  |
| 4   | 0.5      | 1     | 1      | 1       | 1        | 1         |
| 5   | 0.625    | 0.625 | 0.9375 | 0.9375  | 0.9375   | 0.9765625 |
| 6   | 0.75     | 0.75  | 0.75   | 0.9375  | 0.9375   | 0.984375  |
| 7   | 0.875    | 0.875 | 0.875  | 0.875   | 0.984375 | 0.984375  |
| 8   | 1        | 1     | 1      | 1       | 1        | 1         |
| 9   | 0.5625   | 0     | 0.5625 | 0.84375 | 0.984375 | 0.984375  |
| 10  | 0.625    | 0     | 0.625  | 0.9375  | 0.9375   | 0.9375    |
| 11  | 0.6875   | 0     | 0.6875 | 0.6875  | 0.859375 | 0.9453125 |

The complete table is available as a python one-liner:
```py
x = [(n, n/8 if n <= 8 else n/16 if n <= 16 else n/32 if n <= 32 else n/64, 1-(8%n)/8, 1-(16%n)/16, 1-(32%n)/32, 1-(64%n)/64, 1-(128%n)/128) for n in range(1, 64+1)]

```

An `EnumVec8` with 8-bit storage blocks cannot be used to store items larger
than 8 bits. Similarly, for storing elements larger than 32 bits, the default
`EnumVec32` is not enough. The maximum size of an item in bits is defined on
the `EnumLike` crate as the number of bits that can fit in one `usize`.
The `EnumVec` with 128-bit storage is the most memory-efficient option right
now, but most of the operations are 2x slower than the other implementations
on a tipical 64-bit machine. The 8, 16, 32 and 64-bit versions have similar
performance.

The "efficiency limits" of each `EnumVecN`, the largest item size in bits
where it is better than a `Vec` are the following: 

| Storage size | Efficiency limit |
| --- | --- |
| EnumVec8 | 4 |
| EnumVec16 | 4 |
| EnumVec32 | 11 |
| EnumVec64 | 22 |
| EnumVec128 | 42 |

# Customization

To change the default storage just import the `EnumVec` from an internal
module:

```
use enum_vec::vec_u64::EnumVec;
use enum_vec::vec_u8::EnumVec as EnumVec8;
```

This will make the `EnumVec` use 64-bit blocks, improving the memory
efficiency, and also add the option to use an `EnumVec8` with 8-bit blocks.
Note that the `enum_vec![]` macro will always create an `EnumVec`, so code
like:

```
let a: EnumVec8 = enum_vec![];
```

will not compile.

Which storage size to choose?

* Use `EnumVec8` to minimize the overhead of small vectors, well actually
  consider using a `SmallEnumVec` instead.
* Use `EnumVec64` with very large vectors, especially when the element bit size
  is not a power of 2, as it is more memory efficient in some cases.
* Use `EnumVec128` only if memory efficiency is more important than
  performance.
* Use `Vec` if performance is more important than memory efficiency.
* Use `SmallEnumVec` if most of the time you need to store few elements (up to
  128 bits).

# PackedU8

When the item size is 8 or 16 bits, using a `Vec` is always a better option.
But that's not always easy, as a `Vec<[bool; 8]>` will use 8 bytes per element
instead of 8 bits. To force it to use 8 bits wrap it as `Vec<PackedU8<[bool;
8]>>`:

```
use enum_like::PackedU8;

let a = vec![PackedU8::new([true; 8]); 10];

for x in a {
    let x = x.value();
}
```

# SmallEnumVec

There is an experimental `SmallEnumVec` available at:

```
use enum_vec::smallvec_u32::EnumVec as SmallEnumVec;
```

When compiled with the `smallvec` feature, enabled in `Cargo.toml`:

```
enum_vec = { version = "0.3", features = ["smallvec"] }
```

A `SmallEnumVec` will use the stack to store the items, and will only allocate
when it grows too large. The default right now is to use 4x32 bits of inline
storage. This will allow to store 128 1-bit items, 64 2-bit, 32 4-bit, etc.

See the [smallvec](https://github.com/servo/rust-smallvec) crate for more
information.

# Drawbacks

* There is no indexing syntax, since the `EnumVec` can't return a reference.
  Use get and set instead.
* You can't use slice methods, like split(), get(range), reverse(), chunk and
  window iterators, sort(), dedup(), etc. Because there is no deref impl
(unlike `&Vec` which can be used as a `&[T]`).
* Most operations (push, pop, insert, remove) are about 2 or 3 times slower
  than the `Vec` equivalent. Operations like extend, from\_slice, or
  `vec![None; 1000];` are even worse.

# Benchmarks

Here is a comparison of `Vec<T>` vs `EnumVec<T>` when `T` requires 2 bits of storage.

(commit f541c0ed56ad9570fd2291ce42d32980d9831a41)
```
 name                           normal_vec2 ns/iter  enum_vec32_2 ns/iter  diff ns/iter    diff %  speedup 
 ::bench_extend                 604                  7,459                        6,855  1134.93%   x 0.08 
 ::bench_extend_small           75                   193                            118   157.33%   x 0.39 
 ::bench_from_slice             243                  5,499                        5,256  2162.96%   x 0.04 
 ::bench_from_slice_small       45                   130                             85   188.89%   x 0.35 
 ::bench_insert                 11,038               23,645                      12,607   114.21%   x 0.47 
 ::bench_insert_at_zero         26,872               67,778                      40,906   152.23%   x 0.40 
 ::bench_insert_at_zero_small   354                  310                            -44   -12.43%   x 1.14 
 ::bench_insert_small           417                  443                             26     6.24%   x 0.94 
 ::bench_iter_all               3,565                8,140                        4,575   128.33%   x 0.44 
 ::bench_macro_from_elem        93                   3,466                        3,373  3626.88%   x 0.03 
 ::bench_macro_from_elem_small  44                   135                             91   206.82%   x 0.33 
 ::bench_push                   7,678                13,493                       5,815    75.74%   x 0.57 
 ::bench_push_small             291                  251                            -40   -13.75%   x 1.16 
 ::bench_pushpop                7,966                19,530                      11,564   145.17%   x 0.41 
 ::bench_remove                 5,906                16,129                      10,223   173.10%   x 0.37 
 ::bench_remove_at_zero         24,068               107,585                     83,517   347.00%   x 0.22 
 ::bench_remove_at_zero_small   148                  717                            569   384.46%   x 0.21 
 ::bench_remove_small           138                  330                            192   139.13%   x 0.42 
```

The only reason that some benchmarks are faster is because of reallocation:
a `Vec` will reallocate when it reaches 1, 2, 4, 8, ... elements but an
`EnumVec` will reallocate every 32/n, 64/n, ... and since in the benchmark
n=2 and the number of insertions in "\_small" benchmarks defaults to 16,
a `Vec` will reallocate 4 times while an `EnumVec` will reallocate 1 time.

To run the benchmarks yourself, download the source code and run:

```
cargo +nightly bench --features smallvec > bench_log
cargo benchcmp normal_vec2 enum_vec32_2 bench_log
```

You will need to install
[cargo-benchcmp](https://github.com/BurntSushi/cargo-benchcmp)
to be able to easily compare benchmarks.
For example, to compare the default 32-bit `EnumVec` with a 8-bit `EnumVec`,
when dealing with 4-bit elements, run:

```
cargo benchcmp enum_vec32_4 enum_vec8_4 bench bench_log
```

# See also

[enum-set](https://github.com/contain-rs/enum-set)

[enum-map](https://github.com/xfix/enum-map)

[enum-kinds](https://bitbucket.org/Soft/enum-kinds)

[bit-vec](https://github.com/contain-rs/bit-vec)

[smallbitvec](https://github.com/servo/smallbitvec)

[smallvec](https://github.com/servo/rust-smallvec)
