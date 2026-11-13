use crate::*;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{parenthesized, Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, ExprLit, Fields, Index, Lit, LitStr, Member, Meta};

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input: DeriveInput = input.parse().with_message("DeriveInput")?;
        let data = match &input.data {
            Data::Struct(data) => {
                parse_struct_input(data).with_message("failed to parse struct")?
            }
            Data::Enum(data) => {
                parse_enum_input(&input, data).with_message("failed to parse enum")?
            }
            Data::Union(data) => {
                return Err(syn::Error::new(
                    data.union_token.span,
                    "Validator is not supported for unions",
                ))
            }
        };
        Ok(Self {
            vis: input.vis,
            name: input.ident,
            generics: input.generics,
            data,
        })
    }
}

fn parse_enum_input(input: &DeriveInput, data: &DataEnum) -> syn::Result<InputData> {
    let variants: syn::Result<Vec<_>> = data
        .variants
        .iter()
        .map(|variant| {
            let fields = if matches!(variant.fields, Fields::Unit) {
                None
            } else {
                Some(
                    variant
                        .fields
                        .clone()
                        .try_into()
                        .with_message("failed to parse fields")?,
                )
            };
            Ok(EnumVariant {
                derived_type: input.ident.clone(),
                name: variant.ident.clone(),
                fields,
            })
        })
        .collect();
    let variants = variants?;
    Ok(InputData::Enum { variants })
}

impl TryFrom<Fields> for StructFields {
    type Error = syn::Error;
    fn try_from(value: Fields) -> Result<Self, Self::Error> {
        match value {
            Fields::Unit => Err(syn::Error::new(
                value.span(),
                "Validator is not supported for unit structs",
            )),
            Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .cloned()
                    .map(|field| {
                        let vis = field.vis;
                        let name = field.ident.unwrap();
                        let ty = field.ty;
                        let (validator, label) = parse_attrs(field.attrs)
                            .with_message("failed to parse validator from attrs")?;
                        let label = if let Some(label) = label {
                            label.value()
                        } else {
                            name.to_string()
                        };
                        Ok(Field {
                            name: Member::Named(name),
                            ty,
                            vis,
                            validator,
                            label
                        })
                    })
                    .collect::<syn::Result<Vec<_>>>()
                    .with_message("failed to parse fields")?;
                Ok(Self {
                    fields,
                    named_fields: true,
                })
            }
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(i, field)| {
                        let vis = field.vis;
                        let ty = field.ty;
                        let (validator, label) = parse_attrs(field.attrs)
                            .with_message("failed to parse validator from attrs")?;
                        let label = if let Some(label) = label {
                            label.value()
                        } else {
                            i.to_string()
                        };
                        Ok(Field {
                            name: Member::Unnamed(Index::from(i)),
                            ty,
                            vis,
                            validator,
                            label
                        })
                    })
                    .collect::<syn::Result<Vec<_>>>()
                    .with_message("failed to parse fields")?;
                Ok(Self {
                    fields,
                    named_fields: false,
                })
            }
        }
    }
}

fn parse_struct_input(data: &DataStruct) -> syn::Result<InputData> {
    Ok(InputData::Struct {
        fields: data.fields.clone().try_into()?,
        semi_token: data.semi_token,
    })
}

fn parse_attrs(attrs: Vec<Attribute>) -> Result<(Box<dyn Validator>, Option<LitStr>), syn::Error> {
    let mut label = None;
    for attr in &attrs {
        if let Meta::NameValue(list) = &attr.meta {
            if list.path.is_ident("label") {
                let Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) = &list.value else {
                    return Err(syn::Error::new(list.value.span(), "expected string literal"))
                };
                label = Some(value.clone());
            }
        }
    }
    let attr: Vec<_> = attrs
        .iter()
        .filter(|attr| {
            if let Meta::List(list) = &attr.meta {
                list.path.is_ident("validator")
            } else {
                false
            }
        })
        .collect();
    if attr.len() > 1 {
        return Err(syn::Error::new(
            attr[0].span(),
            "validator attribute may only be used once on each field",
        ));
    }
    let Some(&attr) = attr.first() else {
        return Ok((Box::new(Default), label));
    };

    let Meta::List(list) = &attr.meta else {
        return Ok((Box::new(Default), label));
    };
    let validator = Parse::parse
        .parse2(list.tokens.clone())
        .with_message("failed to parse validator")?;
    Ok((validator, label))
}

macro_rules! validator_type_parse {
    ($($validator:ident$(($($arg:tt)*))?),* $(,)?) => {
        $(
        validator_type_parse!(#$validator $(($($arg)*))?);
        )*
    };
    (#$validator:ident) => {
        impl Parse for $validator {
            fn parse(_: ParseStream) -> syn::Result<Self> {
                Ok(Self)
            }
        }
    };
    (#$validator:ident(Punctuated)) => {
        impl Parse for $validator {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let content;
                parenthesized!(content in input);
                let children = Punctuated::parse_separated_nonempty(&content)?;
                Ok(Self(children))
            }
        }
    };
    (#$validator:ident(_)) => {
        impl Parse for $validator {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let content;
                parenthesized!(content in input);
                let arg = content.parse()?;
                Ok(Self(arg))
            }
        }
    };
    (#$validator:ident(_?)) => {
        impl Parse for $validator {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let arg = if input.peek(syn::token::Paren) {
                    let content;
                    parenthesized!(content in input);
                    content.parse()?
                } else {
                    Box::new(Default)
                };
                Ok(Self(arg))
            }
        }
    };
}

validator_type_parse![
    NotEmpty,
    Email,
    Url,
    IpAddr,
    Default,
    Ignore,
    And(Punctuated),
    Or(Punctuated),
    Tuple(Punctuated),
    Length(_),
    Elements(_?),
    Range(_),
    Matches(_),
    ParseAs(_),
    IsNoneOr(_),
    IsSome,
    IsSomeAnd(_?),
];

macro_rules! validator_parse {
    ($($tag:literal => $validator:ident),* $(,)?) => {
        impl Parse for Box<dyn Validator> {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let ident: Ident = input
                    .parse()
                    .with_message("failed to parse validator type")?;
                match ident.to_string().as_str() {
                    $(
                    $tag => Ok(input.parse::<$validator>()?.into_validator()),
                    )*
                    other => Err(syn::Error::new(
                        ident.span(),
                        format!("unknown validator type: \"{other}\""),
                    )),
                }
            }
        }
    };
}

validator_parse!{
    "not_empty" => NotEmpty,
    "and" => And,
    "or" => Or,
    "email" => Email,
    "url" => Url,
    "ip_addr" => IpAddr,
    "length" => Length,
    "elements" => Elements,
    "ignore" => Ignore,
    "tuple" => Tuple,
    "range" => Range,
    "matches" => Matches,
    "parse_as" => ParseAs,
    "option" => IsNoneOr,
    "is_none_or" => IsNoneOr,
    "is_some" => IsSome,
    "is_some_and" => IsSomeAnd
}
