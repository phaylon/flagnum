#![allow(unused_parens)]
#![warn(elided_lifetimes_in_paths)]
#![warn(unused_crate_dependencies)]
#![forbid(unused_must_use)]

//! The procedural macro functionality for the flagnum crate.

use proc_macro::TokenStream;
use syn::{parse_macro_input};


mod parser;
mod builder;

#[proc_macro_attribute]
pub fn flag(attr: TokenStream, item: TokenStream) -> TokenStream {
    builder::FlagnumContext::new(
        parse_macro_input!(attr),
        parse_macro_input!(item),
    ).and_then(|ctx| {
        Ok(ctx.build())
    }).unwrap_or_else(|error| {
        error.to_compile_error()
    }).into()
}
