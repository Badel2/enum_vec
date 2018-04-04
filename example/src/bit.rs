use enum_vec::EnumVec;

#[derive(Copy, Clone, EnumLike)]
enum Bit {
    L,
    H,
    X,
}

pub fn bit_test() {
    let v = vec![Bit::X; 10];
    let _ev = v.into_iter().collect::<EnumVec<Bit>>();
}
