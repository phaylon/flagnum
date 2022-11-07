#![allow(unused_parens)]
#![warn(elided_lifetimes_in_paths)]
#![warn(unused_crate_dependencies)]
#![forbid(unused_must_use)]

use std::collections::HashMap;

// TODO documentation
// TODO tests not covered by documentation
// TODO serde support

use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::{quote};
use syn::punctuated::Punctuated;
use syn::token::{Eq};
use syn::{
    Ident, parse_macro_input, Token, parenthesized, ItemEnum, Error, Expr, Lit, ExprLit,
    parse_str, Type, Visibility, Fields,
};
use syn::parse::{Parse, ParseStream};


#[proc_macro_attribute]
pub fn flag(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ctx = FlagnumContext::new(
        parse_macro_input!(attr),
        parse_macro_input!(item),
    );

    let flag_type = ctx.expand_flag_type();
    let flag_impl = ctx.expand_impls_flag();
    let set_type = ctx.expand_set_type();
    let set_impl = ctx.expand_impls_set();

    quote! {
        #flag_type
        #flag_impl

        #set_type
        #set_impl
    }.into()
}

struct FlagnumContext {
    item: FlagnumEnum,
    name: Ident,
    set_name: Ident,
    repr_ident: Ident,
    repr_type: Type,
    vis: Visibility,
}

impl FlagnumContext {
    fn new(set_name: Ident, item: FlagnumEnum) -> Self {
        let repr_ident = item.repr.to_ident();
        let repr_type = item.repr.to_type();
        let name = item.tree.ident.clone();
        let vis = item.tree.vis.clone();
        Self { item, set_name, name, vis, repr_ident, repr_type }
    }

    fn variant_names(&self) -> impl Iterator<Item = &Ident> + '_ {
        self.item.tree.variants.iter().map(|variant| &variant.ident)
    }

    fn expand_flag_type_serde_derive(&self) -> TokenStream2 {
        if cfg!(feature = "serde") {
            quote! {
                #[derive(
                    flagnum::feature_serde::dep::Deserialize,
                    flagnum::feature_serde::dep::Serialize,
                )]
            }
        } else {
            quote! {}
        }
    }

    fn expand_flag_type(&self) -> TokenStream2 {
        let Self { item: FlagnumEnum { tree, .. }, .. } = self;
        let repr_attr = self.expand_repr_attribute();
        let derive_serde = self.expand_flag_type_serde_derive();
        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #derive_serde
            #repr_attr
            #tree
        }
    }

    fn expand_set_type(&self) -> TokenStream2 {
        let Self { vis, set_name, repr_type, .. } = self;
        let impls_serde = self.expand_set_type_serde_impls();
        quote! {
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
            #vis struct #set_name {
                items: #repr_type,
            }

            #impls_serde
        }
    }

    fn expand_set_type_serde_impls(&self) -> TokenStream2 {
        let Self { set_name, .. } = self;
        if cfg!(feature = "serde") {
            quote! {
                impl<'de> flagnum::feature_serde::dep::Deserialize<'de> for #set_name {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: flagnum::feature_serde::dep::Deserializer<'de>,
                    {
                        deserializer.deserialize_seq(flagnum::feature_serde::SetVisitor::new())
                    }
                }

                impl flagnum::feature_serde::dep::Serialize for #set_name {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: flagnum::feature_serde::dep::Serializer,
                    {
                        use flagnum::feature_serde::dep::ser::SerializeSeq;
                        let mut seq = serializer.serialize_seq(Some(self.len()))?;
                        for item in *self {
                            seq.serialize_element(&item)?;
                        }
                        seq.end()
                    }
                }
            }
        } else {
            quote! {}
        }
    }

    fn expand_repr_attribute(&self) -> TokenStream2 {
        let Self { repr_ident, .. } = self;
        if self.item.tree.variants.empty_or_trailing() {
            quote! {}
        } else {
            quote! {
                #[repr(#repr_ident)]
            }
        }
    }

    fn expand_impls_flag(&self) -> TokenStream2 {
        let Self { name, set_name, .. } = self;
        quote! {
            impl flagnum::Flag for #name {
                type Set = #set_name;
            }
        }
    }

    fn expand_impl_set_shared_fns(&self, is_inherent: bool) -> TokenStream2 {
        let Self { vis, name, repr_type, .. } = self;
        let (prefix, prefix_const) = if is_inherent {
            (quote! { #vis }, quote! { #vis const })
        } else {
            (quote! {}, quote! {})
        };

        quote! {
            #[must_use]
            #[inline(always)]
            #prefix_const fn from_item(item: #name) -> Self {
                Self { items: item as #repr_type }
            }

            #[must_use]
            #[inline(always)]
            #prefix_const fn from_items(mut items: &[#name]) -> Self {
                let mut value = 0;
                while let Some((&first, rest)) = items.split_first() {
                    value |= first as #repr_type;
                    items = rest;
                }
                Self { items: value }
            }

            #[must_use]
            #[inline(always)]
            #prefix_const fn from_sets(mut sets: &[Self]) -> Self {
                let mut value = 0;
                while let Some((&first, rest)) = sets.split_first() {
                    value |= first.items;
                    sets = rest;
                }
                Self { items: value }
            }

            #[must_use]
            #[inline(always)]
            #prefix_const fn len(self) -> usize {
                self.items.count_ones() as usize
            }

            #[must_use]
            #[inline(always)]
            #prefix_const fn is_empty(self) -> bool {
                self.items == 0
            }

            #[must_use]
            #[inline(always)]
            #prefix_const fn is_full(self) -> bool {
                self.items == <Self as Flags>::FULL.items
            }

            #[must_use]
            #prefix fn contains<T>(self, other: T) -> bool
            where
                T: Into<Self>,
            {
                let other = other.into();
                (self.items & other.items) == other.items
            }

            #[must_use]
            #[inline(always)]
            #prefix_const fn intersection(self, other: Self) -> Self {
                Self { items: self.items & other.items }
            }

            #[must_use]
            #[inline(always)]
            #prefix_const fn intersects(self, other: Self) -> bool {
                (self.items & other.items) != 0
            }

            #[must_use]
            #prefix fn with<T>(self, other: T) -> Self
            where
                T: Into<Self>,
            {
                Self { items: self.items | other.into().items }
            }

            #[must_use]
            #prefix fn without<T>(self, other: T) -> Self
            where
                T: Into<Self>,
            {
                Self { items: self.items & !other.into().items }
            }

            #[must_use]
            #[inline(always)]
            #prefix_const fn inverse(self) -> Self {
                Self { items: <Self as Flags>::FULL.items & !self.items }
            }

            #[inline(always)]
            #prefix fn invert(&mut self) {
                self.items = <Self as Flags>::FULL.items & !self.items;
            }

            #prefix fn insert<T>(&mut self, other: T)
            where
                T: Into<Self>,
            {
                self.items |= other.into().items;
            }

            #prefix fn remove<T>(&mut self, other: T)
            where
                T: Into<Self>,
            {
                self.items &= !other.into().items;
            }

            #prefix fn keep<T>(&mut self, other: T)
            where
                T: Into<Self>,
            {
                self.items &= other.into().items;
            }

            #prefix fn retain<F>(&mut self, mut is_retained: F)
            where
                F: FnMut(#name) -> bool,
            {
                for (offset, item) in <Self as Flags>::ITEMS.iter().copied().enumerate() {
                    let item_value = 1 << offset;
                    if (self.items & item_value) != 0 && !is_retained(item) {
                        self.items &= !item_value;
                    }
                }
            }
        }
    }

    fn expand_impls_set(&self) -> TokenStream2 {
        let Self { vis, name, set_name, repr_type, .. } = self;
        let group_consts = self.expand_constant_groups();
        let variant_names = self.variant_names();
        let full_len = self.item.tree.variants.len();

        let fns_inherent = self.expand_impl_set_shared_fns(true);
        let fns_trait = self.expand_impl_set_shared_fns(false);

        quote! {
            impl #set_name {
                #( #group_consts )*

                #[must_use]
                #[inline(always)]
                #vis const fn empty() -> Self {
                    <Self as Flags>::EMPTY
                }

                #[must_use]
                #[inline(always)]
                #vis const fn full() -> Self {
                    <Self as Flags>::FULL
                }

                #[must_use]
                #[inline(always)]
                #vis const fn items() -> &'static [#name] {
                    <Self as Flags>::ITEMS
                }

                #fns_inherent
            }

            impl flagnum::Flags for #set_name {
                type Item = #name;

                const ITEMS: &'static [#name] = &[#( #name::#variant_names ),*];
                const EMPTY: Self = Self { items: 0 };
                const FULL: Self = Self {
                    items: #repr_type::MAX >> (#repr_type::BITS - #full_len as u32),
                };

                #fns_trait
            }

            impl std::fmt::Debug for #set_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_set().entries(self.into_iter()).finish()
                }
            }

            impl From<()> for #set_name {
                fn from(_: ()) -> Self {
                    Self { items: 0 }
                }
            }

            impl From<&#set_name> for #set_name {
                fn from(items: &#set_name) -> Self {
                    *items
                }
            }

            impl From<#name> for #set_name {
                fn from(item: #name) -> Self {
                    Self { items: item as #repr_type }
                }
            }

            impl From<&#name> for #set_name {
                fn from(item: &#name) -> Self {
                    Self { items: *item as #repr_type }
                }
            }

            impl From<Option<#name>> for #set_name {
                fn from(item: Option<#name>) -> Self {
                    if let Some(item) = item {
                        Self { items: item as #repr_type }
                    } else {
                        Self { items: 0 }
                    }
                }
            }

            impl<const N: usize> From<[#name; N]> for #set_name {
                fn from(items: [#name; N]) -> Self {
                    items.into_iter().collect()
                }
            }

            impl From<&[#name]> for #set_name {
                fn from(items: &[#name]) -> Self {
                    items.iter().copied().collect()
                }
            }

            impl From<Vec<#name>> for #set_name {
                fn from(items: Vec<#name>) -> Self {
                    items.into_iter().collect()
                }
            }

            impl From<&Vec<#name>> for #set_name {
                fn from(items: &Vec<#name>) -> Self {
                    items.iter().copied().collect()
                }
            }

            impl FromIterator<#name> for #set_name {
                fn from_iter<I>(iter: I) -> Self
                where
                    I: IntoIterator<Item = #name>,
                {
                    Self {
                        items: iter
                            .into_iter()
                            .map(|item| item as #repr_type)
                            .fold(0, std::ops::BitOr::bitor),
                    }
                }
            }

            impl IntoIterator for #set_name {
                type Item = #name;
                type IntoIter = flagnum::Iter<Self>;

                fn into_iter(self) -> Self::IntoIter {
                    flagnum::Iter::new(self)
                }
            }

            impl IntoIterator for &#set_name {
                type Item = #name;
                type IntoIter = flagnum::Iter<#set_name>;

                fn into_iter(self) -> Self::IntoIter {
                    flagnum::Iter::new(*self)
                }
            }
        }
    }

    fn expand_constant_groups(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        let Self { vis, name, repr_type, .. } = self;
        self.item.groups.iter().map(move |(group, members)| {
            quote! {
                #vis const #group: Self = Self {
                    items: 0 #( | #name::#members as #repr_type )*,
                };
            }
        })
    }
}

struct FlagnumGroups {
    groups: Vec<Ident>,
}

impl Parse for FlagnumGroups {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let arguments;
        parenthesized!(arguments in input);
        let groups: Punctuated<Ident, Token![,]> = arguments.parse_terminated(Ident::parse)?;
        Ok(Self {
            groups: groups.into_iter().collect(),
        })
    }
}

enum FlagnumRepr { U8, U16, U32, U64, U128 }

impl FlagnumRepr {
    fn name(&self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
        }
    }

    fn to_ident(&self) -> Ident {
        parse_str(self.name()).unwrap()
    }

    fn to_type(&self) -> Type {
        parse_str(self.name()).unwrap()
    }
}

struct FlagnumEnum {
    tree: ItemEnum,
    repr: FlagnumRepr,
    groups: HashMap<Ident, Vec<Ident>>,
}

impl Parse for FlagnumEnum {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut item = input.parse::<ItemEnum>()?;

        if item.variants.len() > 128 {
            return Err(Error::new(
                item.enum_token.span,
                "Only up to 128 variants are supported in flagnum enums",
            ));
        }

        let mut set_groups: HashMap<Ident, Vec<Ident>> = HashMap::new();
        let mut max_offset = 0;
        for (offset, variant) in item.variants.iter_mut().enumerate() {
            assert!(offset < 128);
            max_offset = offset;
            let span = variant.ident.span();

            if let Some((eq, _)) = variant.discriminant {
                return Err(Error::new(
                    eq.spans[0],
                    "Unsupported use of explicit discriminant value in flagnum enum variant",
                ));
            }
            match variant.fields {
                Fields::Unit => (),
                _ => {
                    return Err(Error::new(
                        span,
                        "Only unit variants are supported in flagnum enums",
                    ));
                },
            }

            let mut rest_attrs = Vec::new();
            for attr in variant.attrs.iter().cloned() {
                if attr.path.is_ident("groups") {
                    let FlagnumGroups { groups } = syn::parse2(attr.tokens.clone())?;
                    for group in groups {
                        set_groups.entry(group).or_default().push(variant.ident.clone());
                    }
                } else {
                    rest_attrs.push(attr);
                }
            }
            variant.attrs = rest_attrs;

            variant.discriminant = Some((
                Eq { spans: [span] },
                Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: Lit::new(Literal::u128_unsuffixed(1 << offset)),
                }),
            ));
        }

        let repr =
            if max_offset < 8 { FlagnumRepr::U8 }
            else if max_offset < 16 { FlagnumRepr::U16 }
            else if max_offset < 32 { FlagnumRepr::U32 }
            else if max_offset < 64 { FlagnumRepr::U64 }
            else { FlagnumRepr::U128 };

        Ok(Self {
            tree: item,
            repr,
            groups: set_groups,
        })
    }
}
