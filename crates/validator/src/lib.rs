use std::convert::Infallible;
use std::fmt::Debug;
use derive_more::{Deref, From, Into};
#[cfg(feature = "derive")]
pub use ::validator_derive::Validator;

macro_rules! modules {
    ($($module:ident),*) => {
        $(
        mod $module;
        pub use $module::*;
        )*
    };
}

modules!(length, not_empty, and, or, ip, email, url, elements, tuple, range, matches);

pub type ValidationErrors<V> = <<V as Validate>::Validator as Validator<V>>::Error;

pub trait Validate {
    type Validator: Validator<Self>;

    fn validator() -> Self::Validator;
    fn validate(&self) -> Result<(), <Self::Validator as Validator<Self>>::Error> {
        Self::validator().validate(self)
    }
}

pub trait Validator<T: ?Sized>: Sized {
    type Error: Debug;
    fn validate(&self, value: &T) -> Result<(), Self::Error>;
}

#[doc(hidden)]
// this validator always passes values
pub struct IgnoreValidator;

impl ToErrors for Infallible {
    fn build_errors(&self, _: &mut Vec<ErrorString>) {}
}

impl<T> Validator<T> for IgnoreValidator {
    type Error = Infallible;

    fn validate(&self, _: &T) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<V> Validator<String> for V where V: Validator<str> {
    type Error = V::Error;

    fn validate(&self, value: &String) -> Result<(), Self::Error> {
        self.validate(value)
    }
}

impl<V, T> Validator<Vec<T>> for V where V: Validator<[T]> {
    type Error = V::Error;

    fn validate(&self, value: &Vec<T>) -> Result<(), Self::Error> {
        self.validate(value)
    }
}

impl<V, T, const N: usize> Validator<[T; N]> for V where V: Validator<[T]> {
    type Error = V::Error;

    fn validate(&self, value: &[T; N]) -> Result<(), Self::Error> {
        self.validate(value)
    }
}

/// An error set that can output one or more errors, errors will have user-friendly (ish) messages
pub trait ToErrors {
    fn to_errors(&self) -> Vec<ErrorString> {
        let mut errors = vec![];
        self.build_errors(&mut errors);
        errors
    }
    #[doc(hidden)]
    fn build_errors(&self, errors: &mut Vec<ErrorString>);
}

#[derive(Debug, Clone, PartialEq, Into, From, Deref)]
pub struct ErrorString(pub String);

impl<E: ToErrors> ToErrors for Option<E> {
    fn to_errors(&self) -> Vec<ErrorString> {
        if let Some(some) = self {
            some.to_errors()
        } else {
            Vec::new()
        }
    }
    fn build_errors(&self, errors: &mut Vec<ErrorString>) {
        if let Some(some) = self {
            some.build_errors(errors);
        }
    }
}
