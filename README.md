# enum-vec
Efficiently store a vector of enum variants

[![Build Status](https://travis-ci.org/badel2/enum-vec.svg?branch=master)](https://travis-ci.org/badel2/enum-vec)

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

# Memory efficiency
Since internally each block is 32 bits, the `EnumVec` is only 100% memory
efficient when each element is 1, 2, 4, 8, 16 or 32 bits long. The efficiency
in the other cases can be calculated as `1 - (32 % n) / 32`, but it's always
equal or better than a normal `Vec`. However it's equal when n >= 11, so if
you have a type with 2048 variants, you should consider using a `Vec` instead.
```
>>> [(n, n/8 if n <= 8 else n/16 if n <= 16 else n/32, 1-(32 % n)/32) for n in range(1, 32)]
```

n | Vec | EnumVec
--- | --- | ---
1| 0.125| 1.0|
2| 0.25| 1.0|
3| 0.375| 0.9375|
4| 0.5| 1.0|
5| 0.625| 0.9375|
6| 0.75| 0.9375|
7| 0.875| 0.875|
8| 1.0| 1.0|
9| 0.5625| 0.84375|
10| 0.625| 0.9375|

# Usage
Right now the easiest way to try this crate is to clone the entire repository
and add this to your `Cargo.toml`:
```
[dependencies]
enum_vec = { path = "/path/to/enum_vec/enum_vec" }
enum_like = { path = "/path/to/enum_vec/enum_like" }
enum_like_derive = { path = "/path/to/enum_vec/enum_like_derive" }
```

# Example

```rust
#[macro_use]
extern crate enum_like_derive;
extern crate enum_like;
extern crate enum_vec;

use enum_vec::EnumVec;

#[derive(Debug, EnumLike)]
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

# BitVec
Since an EnumVec is essentially a n-bitvec, you can use it as such.
```rust
type BitVec = EnumVec<bool>;
type TwoBitVec = EnumVec<[bool; 2]>;
type TwoBitVec = EnumVec<(bool, bool)>;
type FourBitVec = EnumVec<[bool; 4]>;
```

Unfortunately `FourBitVec` can't be used right now because we are missing
impls for arrays bigger than 3. Ideally you would use your own types instead,
if you want to store a 4-bit digit just impl EnumLike for it:

# impl EnumLike

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

# Drawbacks
* There is no indexing syntax, since the `EnumVec` can't return a reference.
Use get and set instead.
* You can't use slice methods, like split(), get(range), reverse(),
chunk and window iterators, sort(), dedup(), etc. Because there is no deref
impl (unlike `&Vec` which can be used as a `&[T]`).
* Missing impls for element tuples and arrays.

# See also

[enum-set](https://github.com/contain-rs/enum-set)

[enum-map](https://github.com/xfix/enum-map)

[enum-kinds](https://bitbucket.org/Soft/enum-kinds)

[bit-vec](https://github.com/contain-rs/bit-vec)

[smallbitvec](https://github.com/servo/smallbitvec)

