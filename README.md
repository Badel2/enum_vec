# enum-vec
Efficiently store a vector of enum variants

[![Build Status](https://travis-ci.org/Badel2/enum-vec.svg?branch=master)](https://travis-ci.org/Badel2/enum-vec)

[Documentation](https://badel2.github.io/enum-vec/enum_vec/index.html)

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
enum_vec = "0.2"
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

(commit c784168716b45fc661f67d3e6f3f7c623ccc4a53)
```
 name                           normal_vec2 ns/iter  enum_vec32_2 ns/iter  diff ns/iter    diff %  speedup 
 ::bench_extend                 321                  2,325                        2,004   624.30%   x 0.14 
 ::bench_extend_small           45                   88                              43    95.56%   x 0.51 
 ::bench_from_slice             156                  2,142                        1,986  1273.08%   x 0.07 
 ::bench_from_slice_small       26                   58                              32   123.08%   x 0.45 
 ::bench_insert                 5,430                9,972                        4,542    83.65%   x 0.54 
 ::bench_insert_at_zero         11,714               40,131                      28,417   242.59%   x 0.29 
 ::bench_insert_at_zero_small   192                  119                            -73   -38.02%   x 1.61 
 ::bench_insert_small           219                  198                            -21    -9.59%   x 1.11 
 ::bench_iter_all               285                  882                            597   209.47%   x 0.32 
 ::bench_macro_from_elem        30                   2,322                        2,292  7640.00%   x 0.01 
 ::bench_macro_from_elem_small  26                   59                              33   126.92%   x 0.44 
 ::bench_push                   2,877                4,153                        1,276    44.35%   x 0.69 
 ::bench_push_small             151                  83                             -68   -45.03%   x 1.82 
 ::bench_pushpop                2,804                6,182                        3,378   120.47%   x 0.45 
 ::bench_remove                 3,893                9,359                        5,466   140.41%   x 0.42 
 ::bench_remove_at_zero         10,736               66,926                      56,190   523.38%   x 0.16 
 ::bench_remove_at_zero_small   92                   404                            312   339.13%   x 0.23 
 ::bench_remove_small           84                   161                             77    91.67%   x 0.52 
```

The only reason that some benchmarks are faster is because of reallocation:
a `Vec` will reallocate when it reaches 1, 2, 4, 8, ... elements but an
`EnumVec` will reallocate every 32/n, 64/n, ... and since in the benchmark
n=2 and the number of insertions in "\_small" benchmarks defaults to 16,
a `Vec` will reallocate 4 times while an `EnumVec` will reallocate 1 time.

# See also

[enum-set](https://github.com/contain-rs/enum-set)

[enum-map](https://github.com/xfix/enum-map)

[enum-kinds](https://bitbucket.org/Soft/enum-kinds)

[bit-vec](https://github.com/contain-rs/bit-vec)

[smallbitvec](https://github.com/servo/smallbitvec)

[smallvec](https://github.com/servo/rust-smallvec)
