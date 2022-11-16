use proc_macro2::{TokenStream, Literal};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Ident, parse_str, Visibility, Variant, Lit, ExprLit, Expr};
use syn::token::{Eq};

use crate::parser::{FlagnumEnum, FlagnumDecl, WithAttrs};


pub struct FlagnumContext {
    decl: FlagnumDecl,
    body: FlagnumEnum,
    repr_type: Ident,
    item_type: Ident,
    set_type: Ident,
    vis: Visibility,
}

impl FlagnumContext {
    pub fn new(decl: FlagnumDecl, body: FlagnumEnum) -> syn::Result<Self> {
        let Some(repr) = FlagnumRepr::try_from_enum_len(body.item_enum.variants.len()) else {
            return Err(Error::new(
                body.item_enum.enum_token.span,
                "flagnum only supports enums with up to 128 variants",
            ));
        };
        for group in body.grouped.keys() {
            if !decl.groups.iter().any(|decl_group| decl_group.value == *group) {
                return Err(Error::new(
                    group.span(),
                    format!("Undeclared flagnum group `{group}`"),
                ));
            }
        }
        let repr_type = repr.to_ident();
        let item_type = body.item_enum.ident.clone();
        let set_type = decl.set.value.clone();
        let vis = body.item_enum.vis.clone();
        Ok(Self {
            decl,
            body,
            repr_type,
            item_type,
            set_type,
            vis,
        })
    }

    fn variants(&self) -> (usize, impl Iterator<Item = &Ident> + Clone + '_) {
        let variants = &self.body.item_enum.variants;
        (variants.len(), variants.iter().map(|variant| &variant.ident))
    }

    pub fn build(self) -> TokenStream {
        let item = self.build_item_type();
        let set = self.build_set_type();
        quote! {
            #item
            #set
        }
    }

    fn build_item_type(&self) -> TokenStream {
        let Self { repr_type, .. } = self;
        let serde_derive = self.build_item_type_serde_derive();
        let flag_impl = self.build_item_type_flag_impl();
        let mut item = self.body.item_enum.clone();
        for (offset, variant) in item.variants.iter_mut().enumerate() {
            set_variant_offset(variant, offset);
        }
        let repr = if item.variants.is_empty() {
            None
        } else {
            Some(quote! { #[repr(#repr_type)] })
        };
        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #serde_derive
            #repr
            #item
            #flag_impl
        }
    }

    fn build_item_type_flag_impl(&self) -> TokenStream {
        let Self { item_type, set_type, .. } = self;
        quote! {
            impl flagnum::Flag for #item_type {
                type Set = #set_type;
            }
        }
    }

    fn build_item_type_serde_derive(&self) -> Option<TokenStream> {
        if cfg!(feature = "serde") {
            Some(quote! {
                #[derive(
                    flagnum::feature_serde::dep::Deserialize,
                    flagnum::feature_serde::dep::Serialize,
                )]
                #[serde(crate = "flagnum::feature_serde::dep")]
            })
        } else {
            None
        }
    }

    fn build_set_type(&self) -> TokenStream {
        let Self {
            set_type, repr_type,
            decl: FlagnumDecl { set: WithAttrs { attrs, vis, .. }, .. },
            ..
        } = self;
        let serde_impls = self.build_set_type_serde_impls();
        let flags_impl = self.build_set_type_flags_impl();
        let std_trait_impls = self.build_set_type_std_trait_impls();
        let set_type_impl = self.build_set_type_impl();
        quote! {
            #(#attrs)*
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #vis struct #set_type {
                items: #repr_type,
            }
            #set_type_impl
            #flags_impl
            #serde_impls
            #std_trait_impls
        }
    }

    fn build_set_type_impl(&self) -> TokenStream {
        let Self { set_type, .. } = self;
        let common_fns = self.build_set_type_common_const_fns(false);
        let const_groups = self.build_set_type_constant_groups();
        quote! {
            impl #set_type {
                #const_groups
                #common_fns
            }
        }
    }

    fn build_set_type_constant_groups(&self) -> TokenStream {
        let Self {
            repr_type, item_type,
            body: FlagnumEnum { grouped, .. },
            decl: FlagnumDecl { groups, .. },
            ..
        } = self;
        groups.iter().map(|WithAttrs { value: group, attrs, vis }| {
            let members = grouped
                .get(group)
                .map(|members| members.as_slice())
                .unwrap_or_default()
                .iter()
                .map(|member| quote! { #item_type::#member as #repr_type });
            quote! {
                #(#attrs)*
                #vis const #group: Self = Self {
                    items: 0 #( | #members )*,
                };
            }
        }).collect()
    }

    fn build_set_type_std_trait_impls(&self) -> TokenStream {
        let Self { set_type, item_type, repr_type, .. } = self;
        quote! {
            impl std::fmt::Debug for #set_type {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_set().entries(self.into_iter()).finish()
                }
            }

            impl From<()> for #set_type {
                fn from(_: ()) -> Self {
                    Self { items: 0 }
                }
            }

            impl From<#item_type> for #set_type {
                fn from(item: #item_type) -> Self {
                    Self { items: item as #repr_type }
                }
            }

            impl From<Option<#item_type>> for #set_type {
                fn from(item: Option<#item_type>) -> Self {
                    if let Some(item) = item {
                        Self { items: item as #repr_type }
                    } else {
                        Self { items: 0 }
                    }
                }
            }

            impl<const N: usize> From<[#item_type; N]> for #set_type {
                fn from(items: [#item_type; N]) -> Self {
                    items.into_iter().collect()
                }
            }

            impl<const N: usize> From<&[#item_type; N]> for #set_type {
                fn from(items: &[#item_type; N]) -> Self {
                    items.as_slice().into()
                }
            }

            impl From<&[#item_type]> for #set_type {
                fn from(items: &[#item_type]) -> Self {
                    items.iter().copied().collect()
                }
            }

            impl From<Vec<#item_type>> for #set_type {
                fn from(items: Vec<#item_type>) -> Self {
                    items.into_iter().collect()
                }
            }

            impl From<&Vec<#item_type>> for #set_type {
                fn from(items: &Vec<#item_type>) -> Self {
                    items.iter().copied().collect()
                }
            }

            impl FromIterator<#item_type> for #set_type {
                fn from_iter<I>(iter: I) -> Self
                where
                    I: IntoIterator<Item = #item_type>,
                {
                    Self {
                        items: iter
                            .into_iter()
                            .map(|item| item as #repr_type)
                            .fold(0, std::ops::BitOr::bitor),
                    }
                }
            }

            impl IntoIterator for #set_type {
                type Item = #item_type;
                type IntoIter = flagnum::Iter<Self>;

                fn into_iter(self) -> Self::IntoIter {
                    flagnum::Iter::new(self)
                }
            }

            impl IntoIterator for &#set_type {
                type Item = #item_type;
                type IntoIter = flagnum::Iter<#set_type>;

                fn into_iter(self) -> Self::IntoIter {
                    flagnum::Iter::new(*self)
                }
            }

            impl<T> Extend<T> for #set_type
            where
                T: Into<#set_type>,
            {
                fn extend<I>(&mut self, iter: I)
                where
                    I: IntoIterator<Item = T>,
                {
                    for next in iter.into_iter() {
                        self.items |= next.into().items;
                    }
                }
            }
        }
    }

    fn build_set_type_common_const_fns(&self, in_trait: bool) -> TokenStream {
        let Self { vis, item_type, repr_type, .. } = self;
        let with_prefix = |rel_name, body| {
            if in_trait {
                body
            } else {
                let fq_rel_name = format!("flagnum::Flags::{rel_name}");
                quote! {
                    #[doc = "Inherent `const` version of [`"]
                    #[doc = #fq_rel_name]
                    #[doc = "`]."]
                    #[inline(always)]
                    #vis const #body
                }
            }
        };
        TokenStream::from_iter([
            with_prefix("from_item", quote! {
                fn from_item(item: #item_type) -> Self {
                    Self { items: item as #repr_type }
                }
            }),
            with_prefix("from_items", quote! {
                fn from_items(mut items: &[#item_type]) -> Self {
                    let mut value = 0;
                    while let Some((&first, rest)) = items.split_first() {
                        value |= first as #repr_type;
                        items = rest;
                    }
                    Self { items: value }
                }
            }),
            with_prefix("from_sets", quote! {
                fn from_sets(mut sets: &[Self]) -> Self {
                    let mut value = 0;
                    while let Some((&first, rest)) = sets.split_first() {
                        value |= first.items;
                        sets = rest;
                    }
                    Self { items: value }
                }
            }),
            with_prefix("len", quote! {
                fn len(self) -> usize {
                    self.items.count_ones() as usize
                }
            }),
            with_prefix("is_empty", quote! {
                fn is_empty(self) -> bool {
                    self.items == 0
                }
            }),
            with_prefix("is_full", quote! {
                fn is_full(self) -> bool {
                    self.items == <Self as flagnum::Flags>::FULL.items
                }
            }),
        ])
    }

    fn build_set_type_flags_impl(&self) -> TokenStream {
        let Self { set_type, item_type, repr_type, .. } = self;
        let common_fns = self.build_set_type_common_const_fns(true);
        let (_, variants) = self.variants();
        let variants_items = variants.clone();
        quote! {
            impl flagnum::Flags for #set_type {
                type Item = #item_type;

                const EMPTY: Self = Self { items: 0 };
                const FULL: Self = Self {
                    items: 0 #( | #item_type::#variants as #repr_type )*,
                };
                const ITEMS: &'static [#item_type] = &[#( #item_type::#variants_items ),*];

                #common_fns

                fn contains<T>(self, other: T) -> bool
                where
                    T: Into<Self>,
                {
                    let other = other.into();
                    (self.items & other.items) == other.items
                }

                fn overlap<T>(self, other: T) -> Self
                where
                    T: Into<Self>,
                {
                    Self { items: self.items & other.into().items }
                }

                fn has_overlap<T>(self, other: T) -> bool
                where
                    T: Into<Self>,
                {
                    (self.items & other.into().items) != 0
                }

                fn with<T>(self, other: T) -> Self
                where
                    T: Into<Self>,
                {
                    Self { items: self.items | other.into().items }
                }

                fn without<T>(self, other: T) -> Self
                where
                    T: Into<Self>,
                {
                    Self { items: self.items & !other.into().items }
                }

                fn missing(self) -> Self {
                    Self { items: <Self as flagnum::Flags>::FULL.items & !self.items }
                }

                fn invert(&mut self) {
                    self.items = <Self as flagnum::Flags>::FULL.items & !self.items;
                }

                fn insert<T>(&mut self, other: T)
                where
                    T: Into<Self>,
                {
                    self.items |= other.into().items;
                }

                fn remove<T>(&mut self, other: T)
                where
                    T: Into<Self>,
                {
                    self.items &= !other.into().items;
                }

                fn keep<T>(&mut self, other: T)
                where
                    T: Into<Self>,
                {
                    self.items &= other.into().items;
                }

                fn retain<F>(&mut self, mut is_retained: F)
                where
                    F: FnMut(#item_type) -> bool,
                {
                    for (offset, &item) in <Self as flagnum::Flags>::ITEMS.iter().enumerate() {
                        let item_value = 1 << offset;
                        if (self.items & item_value) != 0 && !is_retained(item) {
                            self.items &= !item_value;
                        }
                    }
                }

                fn retained<F>(mut self, is_retained: F) -> Self
                where
                    F: FnMut(#item_type) -> bool,
                {
                    self.retain(is_retained);
                    self
                }
            }
        }
    }

    fn build_set_type_serde_impls(&self) -> Option<TokenStream> {
        let Self { set_type, .. } = self;
        if cfg!(feature = "serde") {
            Some(quote! {
                impl<'de> flagnum::feature_serde::dep::Deserialize<'de> for #set_type {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: flagnum::feature_serde::dep::Deserializer<'de>,
                    {
                        deserializer.deserialize_seq(flagnum::feature_serde::SetVisitor::new())
                    }
                }
                impl flagnum::feature_serde::dep::Serialize for #set_type {
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
            })
        } else {
            None
        }
    }
}

pub enum FlagnumRepr { U8, U16, U32, U64, U128 }

impl FlagnumRepr {
    fn try_from_enum_len(len: usize) -> Option<Self> {
        if len <= 8 { Some(Self::U8) }
        else if len <= 16 { Some(Self::U16) }
        else if len <= 32 { Some(Self::U32) }
        else if len <= 64 { Some(Self::U64) }
        else if len <= 128 { Some(Self::U128) }
        else { None }
    }

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
}

fn set_variant_offset(variant: &mut Variant, offset: usize) {
    let span = variant.span();
    variant.discriminant = Some((
        Eq { spans: [span] },
        Expr::Lit(ExprLit {
            attrs: Vec::new(),
            lit: Lit::new(Literal::u128_unsuffixed(1 << offset)),
        }),
    ));
}