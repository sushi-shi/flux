//! Exact primitive defaults from pinned Rust 8925ea358a0f, core/src/default.rs.
//! Each implementation returns its literal directly; arbitrary Default impls
//! receive no value or panic guarantees from these models.

use flux_attrs::*;

macro_rules! integer_defaults {
    ($($ty:ident),* $(,)?) => { $(
        #[extern_spec(core::default)]
        impl Default for $ty {
            #[no_panic]
            #[spec(fn() -> $ty[0])]
            fn default() -> Self;
        }
    )* };
}

integer_defaults!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

#[extern_spec(core::default)]
impl Default for bool {
    #[no_panic]
    #[spec(fn() -> bool[false])]
    fn default() -> Self;
}
