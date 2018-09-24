#[macro_use]
extern crate enum_like_derive;
extern crate enum_like;
extern crate enum_vec;
extern crate enum_set2;

mod bit;
mod enum_set;

use enum_vec::EnumVec;

#[derive(Copy, Clone, Debug, EnumLike)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

fn main() {
    let mut v = EnumVec::new();
    v.push(Direction::Left);
    v.push(Direction::Right);
    v.push(Direction::Left);
    v.push(Direction::Right);

    // Iterators:
    for d in v.iter() {
        println!("{:?}", d);
    }

    for _ in &v {
        // Same as above
    }
    println!("{:?}", v);

    // This is iter_mut()
    v.for_each(|x| {
        *x = Direction::Up;
    });

    for d in v {
        // Here we take self
        println!("{:?}", d);
    }

    // Error: use of moved value v
    //println!("{}", v.len());

    println!("All possible directions:");
    // All possible directions
    use enum_like::EnumValues;
    for d in Direction::values() {
        println!("{:?}", d);
    }

    bit::bit_test();
}

#[cfg(test)]
mod tests {
    use enum_like::{EnumLike, EnumValues};

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
        // in o(n^2) time because T is not Hash or Cmp or anything
        for i in 0..counter {
            for j in i + 1..counter {
                if seen[i] == seen[j] {
                    panic!("Duplicate entry for {:?}", seen[i]);
                }
            }
        }

        assert_eq!(x, T::NUM_VARIANTS);
    }

    #[test]
    fn unit_types() {
        // All this types should have exactly one variant
        fn chk_ty<T: Clone + ::std::fmt::Debug + PartialEq + EnumLike>() {
            check_values_of::<T>(1);
        }

        chk_ty::<()>();
        chk_ty::<((),)>();
        chk_ty::<((), ())>();
        chk_ty::<((), (), ())>();
        //chk_ty::<((),(),(),(),)>();

        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct A;
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct B {};
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct C();
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum D {
            OneValue,
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum E {
            OneUnnamedValue(D),
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum F {
            OneUnnamedValue(D, E),
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum G {
            OneUnnamedValue(D, E, F),
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum H {
            OneNamedValue { e: E },
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum H2 {
            OneNamedValue { e: E, f: F },
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum H3 {
            OneNamedValue { e: E, f: F, g: G },
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum H4 {
            OneNamedValue { e: E, f: F, g: G, h: H },
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum I {
            ProductOfOneIsOne(A, B, C, D, E, F, G, H),
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct J8(A, B, C, D, E, F, G, H);
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct J4(A, B, C, D);
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct J3(A, B, C);
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct J2(A, B);
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct J1(A);
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct J0();
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct K4 {
            a: A,
            b: B,
            c: C,
            d: D,
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct K3 {
            a: A,
            b: B,
            c: C,
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct K2 {
            a: A,
            b: B,
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct K1 {
            a: A,
        };
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct K0 {};

        chk_ty::<A>();
        chk_ty::<B>();
        chk_ty::<C>();
        chk_ty::<D>();
        chk_ty::<E>();
        chk_ty::<F>();
        chk_ty::<G>();
        chk_ty::<H>();
        chk_ty::<H2>();
        chk_ty::<H3>();
        chk_ty::<H4>();
        chk_ty::<I>();
        chk_ty::<J0>();
        chk_ty::<J1>();
        chk_ty::<J2>();
        chk_ty::<J3>();
        chk_ty::<J4>();
        chk_ty::<K0>();
        chk_ty::<K1>();
        chk_ty::<K2>();
        chk_ty::<K3>();
        chk_ty::<K4>();
    }

    #[test]
    fn never_types() {
        // All this types should have exactly zero variants
        fn chk_ty<T: Clone + ::std::fmt::Debug + PartialEq + EnumLike>() {
            check_values_of::<T>(0);
        }

        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum Empty {}

        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum EmptyVar {
            A(Empty),
            B(Empty, Empty),
            C { e: Empty },
            D { e: Empty, d: Empty },
        }

        chk_ty::<Empty>();
        // This used to issue two warnings: % 0 and / 0
        // which should be fine because we never call T::from_discr() on these
        // types
        // A possible optimization would be to just return 0; in to_discr(),
        // but the proc macro doesn't know the value of T::NUM_VARIANTS
        // Maybe we can add assert!(T::NUM_VARIANTS != 0) and hope the compiler
        // will optimize it out?
        chk_ty::<(Empty, Empty)>();
        chk_ty::<(Empty, Empty, Empty)>();
        chk_ty::<EmptyVar>();

        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum OneVar {
            A(Empty),
            B(Empty, Empty),
            C { e: Empty },
            D { e: Empty, d: Empty },
            E,
        }
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum TwoVar {
            A(Empty),
            A1,
            B(Empty, Empty),
            C { e: Empty },
            D { e: Empty, d: Empty },
            E,
        }
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum ManyVar {
            A(Empty),
            A1(bool), // 2
            B(Empty, Empty),
            C { e: Empty },
            D { e: Empty, d: Empty },
            E(TwoVar), // 2
        }
        check_values_of::<OneVar>(1);
        check_values_of::<TwoVar>(2);
        check_values_of::<ManyVar>(2 + 2);
    }

    // Tests for derive(EnumLike)
    #[test]
    fn derive_clike_enum() {
        // Simple enums, these are a special case in enum_like_derive,
        // so they are probably bug-free
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum One {
            A,
        }
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum Two {
            A = 100,
            B,
        }
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum Filey {
            Read = 4,
            Write = 2,
            Exec = 1,
        }
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum Booly {
            True,
            False,
            Truey,
            Falsey,
        }

        check_values_of::<One>(1);
        check_values_of::<Two>(2);
        check_values_of::<Filey>(3);
        check_values_of::<Booly>(4);
    }

    #[test]
    fn derive_rust_enum() {
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum Opt {
            Not,
            Maybe,
            Probably,
            Always { x0: bool, x1: bool, x2: bool },
            MeToo(bool, bool),
        }

        check_values_of::<Opt>(3 + 2 * 2 * 2 + 2 * 2);
    }

    #[test]
    #[should_panic]
    fn from_discr_invalid_value() {
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum Filey {
            Read = 4,
            Write = 2,
            Exec = 1,
        }
        let _read = Filey::from_discr(4);
    }

    #[test]
    #[should_panic]
    fn invalid_impl() {
        #[derive(Copy, Clone, Debug, PartialEq)]
        enum BadOption<T> {
            Non,
            Som(T),
        }
        unsafe impl<T: EnumLike> EnumLike for BadOption<T> {
            const NUM_VARIANTS: usize = 1 + T::NUM_VARIANTS;
            fn to_discr(self) -> usize {
                match self {
                    BadOption::Non => 0,
                    BadOption::Som(x) => 0 + x.to_discr(),
                }
            }
            fn from_discr(x: usize) -> Self {
                match x {
                    0 => BadOption::Non,
                    x => BadOption::Som(T::from_discr(x)),
                }
            }
        }

        // Non and Som(false) should both map to 0, this is an error
        check_values_of::<BadOption<bool>>(3);
    }

    #[test]
    fn generics() {
        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        enum GoodOption<T> {
            Non,
            Som(T),
        }

        check_values_of::<GoodOption<bool>>(3);

        #[derive(Copy, Clone, Debug, PartialEq, EnumLike)]
        struct BoolTS<T, S>(bool, T, S);
        check_values_of::<BoolTS<(), Option<()>>>(2 * 1 * 2);
    }
}
