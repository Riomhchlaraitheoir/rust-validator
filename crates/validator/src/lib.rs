use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use derive_more::{Deref, From, Into};
#[cfg(feature = "derive")]
pub use ::validator_derive::Validator;

pub mod length;
pub mod not_empty;
pub mod and;
pub mod or;
pub mod ip;
pub mod email;
pub mod url;
pub mod elements;
pub mod tuple;
pub mod range;
pub mod matches;
pub mod parse_as;
pub mod option;

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
pub struct IgnoreValidator<T>(PhantomData<T>);

impl ToErrors for ! {
    fn build_errors(&self, _: &mut Vec<ErrorString>) {}
}

#[allow(clippy::new_without_default)]
impl<T> IgnoreValidator<T> {
    pub fn new() -> Self {
        IgnoreValidator(PhantomData)
    }
}

pub struct IgnoreError<T> {
    _t: PhantomData<T>,
    error: !
}

impl<T> PartialEq for IgnoreError<T> {
    fn eq(&self, _: &Self) -> bool {
        self.error
    }
}

impl<T> Clone for IgnoreError<T> {
    fn clone(&self) -> Self {
        self.error
    }
}

impl<T> ToErrors for IgnoreError<T> {
    fn build_errors(&self, _: &mut Vec<ErrorString>) {
        self.error
    }
}

impl<T> From<IgnoreError<T>> for ! {
    fn from(error: IgnoreError<T>) -> ! {
        error.error
    }
}

impl<T> Debug for IgnoreError<T> {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        self.error
    }
}

impl<T> Validator<T> for IgnoreValidator<T> {
    type Error = IgnoreError<T>;

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
