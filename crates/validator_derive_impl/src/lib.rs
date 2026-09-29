use proc_macro2::{Ident, TokenStream};
use syn::token::Semi;
use syn::{
    ExprRange

    , Member

    , Type, Visibility,
};

#[cfg(test)]
mod test;
mod parse;
mod output;

pub fn derive(input: Input) -> TokenStream {
    input.generate_output()
}

pub struct Input {
    vis: Visibility,
    name: Ident,
    data: InputData,
}

enum InputData {
    Struct {
        fields: StructFields,
        semi_token: Option<Semi>,
    },
    Enum {
        variants: Vec<EnumVariant>,
    },
}

struct StructFields {
    fields: Vec<Field>,
    named_fields: bool,
}

struct EnumVariant {
    derived_type: Ident,
    name: Ident,
    fields: Option<StructFields>,
}

struct Field {
    name: Member,
    ty: Type,
    vis: Visibility,
    validator: Validator,
    label: String
}

#[derive(Default)]
enum Validator {
    NotEmpty,
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Email,
    Url,
    IpAddr,
    Length(Option<usize>, Option<usize>),
    Elements(Box<Self>),
    #[default]
    Default,
    Ignore,
    Tuple(Vec<Self>),
    Range(ExprRange),
    Matches(Member),
    ParseAs(Box<Type>)
}

trait WithMessage {
    fn with_message(self, msg: &str) -> Self;
}

impl<T> WithMessage for syn::Result<T> {
    #[cfg(test)]
    fn with_message(self, msg: &str) -> Self {
        self.map_err(|err| syn::Error::new(err.span(), format!("{msg}: {err}")))
    }
    #[cfg(not(test))]
    fn with_message(self, _msg: &str) -> Self {
        self
    }
}
