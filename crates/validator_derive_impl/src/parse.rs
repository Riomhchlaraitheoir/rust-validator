use crate::*;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{parenthesized, token, Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, ExprLit, ExprRange, Fields, Index, Lit, LitInt, LitStr, Member, Meta, Token};

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

fn parse_attrs(attrs: Vec<Attribute>) -> Result<(Validator, Option<LitStr>), syn::Error> {
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
        return Ok((Validator::default(), label));
    };

    let Meta::List(list) = &attr.meta else {
        return Ok((Validator::default(), label));
    };
    let validator = Validator::parse
        .parse2(list.tokens.clone())
        .with_message("failed to parse validator")?;
    Ok((validator, label))
}

impl Parse for Validator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input
            .parse()
            .with_message("failed to parse validator type")?;
        let val_type = ident.to_string();
        match val_type.as_str() {
            "not_empty" => Ok(Validator::NotEmpty),
            "email" => Ok(Validator::Email),
            "url" => Ok(Validator::Url),
            "ip" => Ok(Validator::IpAddr),
            "ignore" => Ok(Validator::Ignore),
            "elements" => Ok(Validator::Elements({
                Box::new(if input.peek(token::Paren) {
                    let content;
                    parenthesized!(content in input);
                    content.parse()?
                } else {
                    Validator::Default
                })
            })),
            "length" => {
                let content;
                parenthesized!(content in input);
                let mut equal_name = None;
                let mut equal = None;
                let mut min = None;
                let mut max = None;
                loop {
                    let name: Ident = content
                        .parse()
                        .with_message("failed to parse length option name")?;
                    content
                        .parse::<Token![=]>()
                        .with_message("failed to parse length option '=' token")?;
                    let value: LitInt = content
                        .parse()
                        .with_message("failed to parse length option value")?;
                    let value: usize = value.base10_parse().map_err(|err| {
                        syn::Error::new(value.span(), format!("failed to parse usize: {err}"))
                    })?;
                    match name.to_string().as_str() {
                        "equal" => {
                            equal_name = Some(name);
                            equal = Some(value)
                        }
                        "min" => min = Some(value),
                        "max" => max = Some(value),
                        other => {
                            return Err(syn::Error::new(
                                name.span(),
                                format!(r#"unknown option: "{other}""#),
                            ))
                        }
                    }
                    if Comma::parse(&content).is_err() {
                        break;
                    }
                }

                if let Some(equal) = equal {
                    if min.is_some() || max.is_some() {
                        return Err(syn::Error::new(
                            equal_name.unwrap().span(),
                            "cannot use 'equal' with either 'min' or 'max'",
                        ));
                    }

                    Ok(Validator::Length(Some(equal), Some(equal)))
                } else if min.is_some() || max.is_some() {
                    Ok(Validator::Length(min, max))
                } else {
                    Err(syn::Error::new(
                        val_type.span(),
                        "no options found, one of 'equal', 'min', 'max' must be set",
                    ))
                }
            }
            "and" | "or" => {
                let content;
                parenthesized!(content in input);
                let left = content
                    .parse()
                    .with_message("failed to parse binary left")?;
                let _ = content
                    .parse::<Token![,]>()
                    .with_message("failed to parse binary comma")?;
                let right = content
                    .parse()
                    .with_message("failed to parse binary right")?;
                Ok(match val_type.as_str() {
                    "and" => Validator::And(Box::new(left), Box::new(right)),
                    "or" => Validator::Or(Box::new(left), Box::new(right)),
                    _ => unreachable!(),
                })
            }
            "tuple" => {
                let content;
                parenthesized!(content in input);
                let mut children = Vec::new();
                loop {
                    let child = content.parse().with_message(&format!(
                        "failed to parse tuple child {i}",
                        i = children.len()
                    ))?;
                    children.push(child);
                    if Comma::parse(&content).is_err() {
                        break;
                    }
                }
                Ok(Validator::Tuple(children))
            }
            "range" => {
                let content;
                parenthesized!(content in input);
                let range: ExprRange = content.parse()?;
                Ok(Validator::Range(range))
            }
            "matches" => {
                let content;
                parenthesized!(content in input);
                let other = content.parse()?;
                Ok(Validator::Matches(other))
            }
            other => Err(syn::Error::new(
                ident.span(),
                format!("unknown validator type: \"{other}\""),
            )),
        }
    }
}
