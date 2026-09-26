use quote::quote;
use syn::File;
use syn::parse::{Parse, Parser};
use crate::Input;

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

tests![struct_validator, tuple_validator, enum_validator, list_validator];

fn run_test(input: &str, expected: &str) {
    let input: Input = syn::parse_str(input).unwrap();
    let expected: File = syn::parse_str(expected).unwrap();
    let output = File::parse.parse2(super::derive(input)).expect("failed to parse output");
    let expected = prettyplease::unparse(&expected);
    let output = prettyplease::unparse(&output);
    difference::assert_diff!(&output, &expected, "\n", 0);
}

