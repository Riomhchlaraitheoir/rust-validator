use quote::quote;
use syn::DeriveInput;
use crate::Input;

#[test]
fn struct_validator() {
    let input = quote! {
        struct SignupData {
            #[validator(email)]
            mail: String,
            #[validator(url)]
            site: String,
            #[validator(length(min = 1))]
            first_name: String,
            #[validator(range(18..))]
            age: u8,
            #[validator(elements)]
            dogs: Vec<Dog>
        }
    };

    let input: Input = syn::parse2(input).unwrap_or_else(|err| panic!("failed to parse input: {err}: {start:?} {end:?} ", start = err.span().start(), end = err.span().end()));
    let output = super::derive(input);
    let as_file = syn::parse_file(&output.to_string())
        .unwrap_or_else(|err| panic!("failed to parse outputted code: {err}\n{}", &output.to_string()));
    let formatted = prettyplease::unparse(&as_file);
    insta::assert_snapshot!(formatted)
}

#[test]
fn tuple_validator() {
    let input = quote! {
        struct SignupData(
            #[validator(email)]
            String,
            #[validator(url)]
            String,
            #[validator(length(min = 1))]
            String,
        );
    };

    let input: Input = syn::parse2(input).unwrap_or_else(|err| panic!("failed to parse input: {err}: {start:?} {end:?} ", start = err.span().start(), end = err.span().end()));
    let output = super::derive(input);
    let as_file = syn::parse_file(&output.to_string())
        .unwrap_or_else(|err| panic!("failed to parse outputted code: {err}\n{}", &output.to_string()));
    let formatted = prettyplease::unparse(&as_file);
    insta::assert_snapshot!(formatted)
}
#[test]
fn enum_validator() {
    let input = quote! {
        enum Request {
            Signup {
                #[validator(email)]
                mail: String,
                #[validator(url)]
                site: String,
                #[validator(length(min = 1))]
                first_name: String,
            },
            Login(#[validator(email)] String, #[validator(length(min = 8, max = 64))] String),
            Logout
        }
    };

    let input: Input = syn::parse2(input).unwrap_or_else(|err| panic!("failed to parse input: {err}: {start:?} {end:?} ", start = err.span().start(), end = err.span().end()));
    let output = super::derive(input);
    let as_file = syn::parse_file(&output.to_string())
        .unwrap_or_else(|err| panic!("failed to parse outputted code: {err}\n{}", &output.to_string()));
    let formatted = prettyplease::unparse(&as_file);
    insta::assert_snapshot!(formatted)
}


#[test]
fn list_validator() {
    let input = quote! {
        struct HasList {
            #[validator(elements)]
            list: Vec<Element>
        }
    };

    let input: Input = syn::parse2(input).unwrap_or_else(|err| panic!("failed to parse input: {err}: {start:?} {end:?} ", start = err.span().start(), end = err.span().end()));
    let output = super::derive(input);
    let as_file = syn::parse_file(&output.to_string())
        .unwrap_or_else(|err| panic!("failed to parse outputted code: {err}\n{}", &output.to_string()));
    let formatted = prettyplease::unparse(&as_file);
    insta::assert_snapshot!(formatted)
}


