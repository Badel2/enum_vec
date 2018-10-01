# enum-vec
Efficiently store a vector of enum variants

[![Crates.io](https://img.shields.io/crates/v/enum_vec.svg)](https://crates.io/crates/enum_vec)
[![License GPLv3](https://img.shields.io/github/license/badel2/enum_vec.svg)](https://github.com/Badel2/enum_vec/blob/master/LICENSE)
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

(commit e8db9c883b82e472e9aefb6087be55dafd76b6a0)
```
 name                           normal_vec2 ns/iter  enum_vec32_2 ns/iter  diff ns/iter    diff %  speedup 
 ::bench_all                    3                    5                                2    66.67%   x 0.60 
 ::bench_all_small              3                    5                                2    66.67%   x 0.60 
 ::bench_all_worst_case         1,308                41                          -1,267   -96.87%  x 31.90 
 ::bench_all_worst_case_small   19                   5                              -14   -73.68%   x 3.80 
 ::bench_any                    8                    12                               4    50.00%   x 0.67 
 ::bench_any_small              8                    12                               4    50.00%   x 0.67 
 ::bench_any_worst_case         447                  59                            -388   -86.80%   x 7.58 
 ::bench_any_worst_case_small   11                   6                               -5   -45.45%   x 1.83 
 ::bench_extend                 419                  3,793                        3,374   805.25%   x 0.11 
 ::bench_extend_small           48                   108                             60   125.00%   x 0.44 
 ::bench_from_slice             180                  3,237                        3,057  1698.33%   x 0.06 
 ::bench_from_slice_small       27                   79                              52   192.59%   x 0.34 
 ::bench_insert                 8,059                13,154                       5,095    63.22%   x 0.61 
 ::bench_insert_at_zero         16,898               38,729                      21,831   129.19%   x 0.44 
 ::bench_insert_at_zero_small   218                  190                            -28   -12.84%   x 1.15 
 ::bench_insert_small           275                  258                            -17    -6.18%   x 1.07 
 ::bench_iter_all               2,327                4,948                        2,621   112.63%   x 0.47 
 ::bench_macro_from_elem        602                  2,435                        1,833   304.49%   x 0.25 
 ::bench_macro_from_elem_small  28                   80                              52   185.71%   x 0.35 
 ::bench_push                   4,914                7,097                        2,183    44.42%   x 0.69 
 ::bench_push_small             181                  130                            -51   -28.18%   x 1.39 
 ::bench_pushpop                4,390                12,107                       7,717   175.79%   x 0.36 
 ::bench_remove                 5,261                10,823                       5,562   105.72%   x 0.49 
 ::bench_remove_at_zero         15,880               68,593                      52,713   331.95%   x 0.23 
 ::bench_remove_at_zero_small   101                  443                            342   338.61%   x 0.23 
 ::bench_remove_small           103                  207                            104   100.97%   x 0.50 
```

The only methods which are definitely faster than the `Vec` equivalent are
`all` and `any`, which take advantage of the packing to process many
elements at once.
Some other benchmarks appear faster because of reallocation:
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
