use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(Validator, attributes(validator))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    validator_derive_impl::derive(input).into()
}