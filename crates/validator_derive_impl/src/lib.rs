use std::any::Any;
use proc_macro2::{Ident, TokenStream};
use syn::token::Semi;
use syn::{ExprRange, Generics, Member, Token, Type, Visibility};
use syn::punctuated::Punctuated;
use crate::output::ValidatorOutput;

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
    generics: Generics,
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
    validator: Box<dyn Validator>,
    label: String
}

trait Validator: ValidatorOutput + Any {
    #[cfg(test)]
    fn equal_any(&self, other: &dyn Any) -> bool;
    #[cfg(test)]
    fn equal(&self, other: &dyn Validator) -> bool {
        self.equal_any(other.as_any())
    }
    #[cfg(test)]
    fn as_any(&self) -> &dyn Any;
}

struct NotEmpty;
struct Email;
struct Url;
struct IpAddr;
struct Default;
struct Ignore;
struct And(Punctuated<Box<dyn Validator>, Token![,]>);
struct Or(Punctuated<Box<dyn Validator>, Token![,]>);
struct Length(ExprRange);
struct Elements(Box<dyn Validator>);
struct Tuple(Punctuated<Box<dyn Validator>, Token![,]>);
struct Range(ExprRange);
struct Matches(Member);
struct ParseAs(Box<Type>);
struct IsNoneOr(Box<dyn Validator>);
struct IsSome;
struct IsSomeAnd(Box<dyn Validator>);

trait IntoValidator {
    fn into_validator(self) -> Box<dyn Validator>;
}

impl<V: Validator> IntoValidator for V {
    fn into_validator(self) -> Box<dyn Validator> {
        Box::new(self)
    }
}

impl IntoValidator for IsSome {
    fn into_validator(self) -> Box<dyn Validator> {
        IsSomeAnd(Ignore.into_validator()).into_validator()
    }
}


macro_rules! impl_validator {
    ($($validator:ident$(($($arg:tt)*))?),* $(,)?) => {
        $(
        impl Validator for $validator {
            #[cfg(test)]
            fn as_any(&self) -> &dyn Any {
                self
            }
            impl_validator!(#$validator $(($($arg)*))?);
        }
        )*
    };
    (#$validator:ident) => {
        #[cfg(test)]
        fn equal_any(&self, other: &dyn Any) -> bool {
            let Self = self;
            other.type_id() == self.type_id()
        }
    };
    (#$validator:ident(Punctuated<Box<dyn Validator>, $token:ty>)) => {
        #[cfg(test)]
        fn equal_any(&self, other: &dyn Any) -> bool {
            let Some(other) = other.downcast_ref::<Self>() else { return false };
            self.0.iter().zip(other.0.iter()).all(|(self_, other)| self_.equal(other.as_ref()))
        }
    };
    (#$validator:ident(Box<dyn Validator>)) => {
        #[cfg(test)]
        fn equal_any(&self, other: &dyn Any) -> bool {
            use std::ops::Deref;
            let Some(other) = other.downcast_ref::<Self>() else { return false };
            self.0.equal(other.0.deref())
        }
    };
    (#$validator:ident(_)) => {
        #[cfg(test)]
        fn equal_any(&self, other: &dyn Any) -> bool {
            let Some(other) = other.downcast_ref::<Self>() else { return false };
            self.0 == other.0
        }
    };
}

impl_validator![
    NotEmpty,
    Email,
    Url,
    IpAddr,
    Default,
    Ignore,
    And(Punctuated<Box<dyn Validator>, Token![,]>),
    Or(Punctuated<Box<dyn Validator>, Token![,]>),
    Tuple(Punctuated<Box<dyn Validator>, Token![,]>),
    Length(_),
    Elements(Box<dyn Validator>),
    Range(_),
    Matches(_),
    ParseAs(_),
    IsNoneOr(Box<dyn Validator>),
    IsSomeAnd(Box<dyn Validator>)
];

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
