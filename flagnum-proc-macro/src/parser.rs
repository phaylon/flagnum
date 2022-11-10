use std::collections::HashMap;

use syn::punctuated::Punctuated;
use syn::{Attribute, parenthesized, Token, Ident, ItemEnum, Fields, Error, parse2};
use syn::parse::{Parse, ParseStream};


mod kw {
    use syn::custom_keyword;
    custom_keyword!(groups);
}

pub struct FlagnumDecl {
    pub set: WithAttrs<Ident>,
    pub groups: Vec<WithAttrs<Ident>>,
}

impl Parse for FlagnumDecl {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let set = input.parse()?;
        let mut groups = Vec::new();
        let mut groups_initialized = false;
        while input.call(try_parse_comma_continuation)? {
            if let Some(groups_decl) = input.call(try_parse_groups_decl)? {
                if groups_initialized {
                    return Err(input.error(
                        "Groups have already been declared for this flagnum enum",
                    ));
                } else {
                    groups_initialized = true;
                }
                groups = groups_decl;
                continue;
            } else {
                let error_msg = if groups_initialized {
                    "Expected no further arguments"
                } else {
                    "Expected a `groups` declaration or the end of arguments"
                };
                return Err(input.error(error_msg));
            }
        }
        Ok(Self {
            set,
            groups,
        })
    }
}

pub struct FlagnumEnum {
    pub item_enum: ItemEnum,
    pub grouped: HashMap<Ident, Vec<Ident>>,
}

impl Parse for FlagnumEnum {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut item_enum: ItemEnum = input.parse()?;
        let mut grouped: HashMap<Ident, Vec<Ident>> = HashMap::new();
        for variant in &mut item_enum.variants {
            if let Some((eq, _)) = variant.discriminant {
                return Err(Error::new(
                    eq.spans[0],
                    "flagnum enums cannot have explicit discriminant values",
                ));
            }
            match variant.fields {
                Fields::Unit => (),
                _ => {
                    return Err(Error::new(
                        variant.ident.span(),
                        "flagnum enums can only contain unit variants",
                    ));
                },
            }
            let mut retained_attrs = Vec::new();
            for attr in &variant.attrs {
                if attr.path.is_ident("groups") {
                    let variant_groups: Arguments<Ident> = parse2(attr.tokens.clone())?;
                    for group in variant_groups.values {
                        grouped.entry(group).or_default().push(variant.ident.clone());
                    }
                } else {
                    retained_attrs.push(attr.clone());
                }
            };
            variant.attrs = retained_attrs;
        }
        Ok(Self {
            item_enum,
            grouped,
        })
    }
}

fn try_parse_comma_continuation(input: ParseStream<'_>) -> syn::Result<bool> {
    if input.peek(Token![,]) {
        let _: Token![,] = input.parse()?;
        Ok(!input.is_empty())
    } else {
        if input.is_empty() {
            Ok(false)
        } else {
            Err(input.error("Expected a comma before the next list element"))
        }
    }
}

fn try_parse_groups_decl<T>(input: ParseStream<'_>) -> syn::Result<Option<Vec<T>>>
where
    T: Parse,
{
    if !input.peek(kw::groups) {
        return Ok(None);
    }
    parse_groups_decl(input)
}

fn parse_groups_decl<T>(input: ParseStream<'_>) -> syn::Result<Option<Vec<T>>>
where
    T: Parse,
{
    let _: kw::groups = input.parse()?;
    let groups = input.call(parse_arguments)?;
    Ok(Some(groups))
}

struct Arguments<T> {
    values: Vec<T>,
}

impl<T> Parse for Arguments<T>
where
    T: Parse,
{
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let values = parse_arguments(input)?;
        Ok(Self { values })
    }
}

fn parse_arguments<T>(input: ParseStream<'_>) -> syn::Result<Vec<T>>
where
    T: Parse,
{
    let arguments;
    parenthesized!(arguments in input);
    let items: Punctuated<T, Token![,]> = arguments.parse_terminated(T::parse)?;
    Ok(items.into_iter().collect())
}

pub struct WithAttrs<T> {
    pub value: T,
    pub attrs: Vec<Attribute>,
}

impl<T> Parse for WithAttrs<T>
where
    T: Parse,
{
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let value = input.parse()?;
        Ok(Self { value, attrs })
    }
}