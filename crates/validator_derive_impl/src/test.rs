use crate::Input;
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::parse::{Parse, Parser};
use syn::{Attribute, File, Item, Meta};

macro_rules! tests {
    ($($name:ident),*) => {
        $(
        #[test]
        fn $name() {
            let input = include_str!(concat!("test/inputs/", stringify!($name), ".rs"));
            let output = include_str!(concat!("test/outputs/", stringify!($name), ".rs"));
            run_test(input, output)
        }
        )*
    };
}

tests![struct_validator, tuple_validator, enum_validator, list_validator, password_validator];

fn run_test(input: &str, expected: &str) {
    let input: File = syn::parse_str(input).unwrap();
    let mut output = TokenStream::new();
    for item in input.items {
        let Some(input) = parse_derive_input(item) else {
            continue;
        };
        output.extend(super::derive(input.expect("failed to parse input")));
    }
    let expected: File = syn::parse_str(expected).unwrap();
    let output = match File::parse.parse2(output.clone()) {
        Ok(output) => output,
        Err(err) => {
            println!("{output}");
            panic!("failed ot parse output: {err}");
        },
    };
    let expected = prettyplease::unparse(&expected);
    let output = prettyplease::unparse(&output);
    difference::assert_diff!(&output, &expected, "\n", 0);
}

fn parse_derive_input(item: Item) -> Option<syn::Result<Input>> {
    match item {
        Item::Struct(item) if has_derive_attr(&item.attrs) => {
            Some(Input::parse.parse2(item.into_token_stream()))
        }
        Item::Enum(item) if has_derive_attr(&item.attrs) => {
            Some(Input::parse.parse2(item.into_token_stream()))
        }
        _ => None
    }
}

fn has_derive_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        let Meta::List(list) = &attr.meta else {
            continue;
        };
        if !list.path.is_ident("derive") {
            continue;
        }
        for token in list.tokens.clone() {
            let TokenTree::Ident(ident) = token else {
                continue;
            };
            if ident == "Validator" {
                return true;
            }
        }
    }
    false
}
