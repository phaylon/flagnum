#![allow(unused_parens)]
#![warn(elided_lifetimes_in_paths)]
#![warn(unused_crate_dependencies)]
#![forbid(unused_must_use)]

pub use flagnum_proc_macro::*;

pub use flagnum_core::{
    Flag,
    Flags,
};
