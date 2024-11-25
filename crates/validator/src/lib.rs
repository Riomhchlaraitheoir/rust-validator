use std::convert::Infallible;
use std::fmt::Debug;

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

modules!(not_empty, and, or, email, url, ip, length, elements, tuple, range);

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
