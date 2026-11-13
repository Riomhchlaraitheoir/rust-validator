use thiserror::Error;
use crate::{ErrorString, ToErrors, Validator};

pub trait OptionValue {
    type Inner;
    fn inner(&self) -> Option<&Self::Inner>;
}

impl<T> OptionValue for Option<T> {
    type Inner = T;
    fn inner(&self) -> Option<&T> {
        self.as_ref()
    }
}

/// Validates the value if it is present
pub struct NoneOrValidator<V> {
    inner: V,
}

impl<V> NoneOrValidator<V> {
    pub fn new(inner: V) -> Self {
        NoneOrValidator { inner }
    }
}

impl<V: Validator<T::Inner>, T: OptionValue> Validator<T> for NoneOrValidator<V> {
    type Error = V::Error;
    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        if let Some(value) = value.inner() {
            self.inner.validate(value)
        } else {
            Ok(())
        }
    }
}


pub struct SomeAndValidator<V> {
    field: &'static str,
    inner: V
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum SomeAndError<E> {
    #[error(transparent)]
    Some(E),
    #[error("Is None")]
    IsNone(&'static str),
}

impl<E: ToErrors> ToErrors for SomeAndError<E> {
    fn build_errors(&self, errors: &mut Vec<ErrorString>) {
        match self {
            SomeAndError::Some(error) => error.build_errors(errors),
            SomeAndError::IsNone(field) => {
                errors.push(format!("{field} is required").into())
            }
        };
    }
}

impl<V> SomeAndValidator<V> {
    pub fn new(field: &'static str, inner: V) -> Self {
        SomeAndValidator { field, inner }
    }
}

impl<T: OptionValue, V: Validator<T::Inner>> Validator<T> for SomeAndValidator<V> {
    type Error = SomeAndError<V::Error>;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        match value.inner() {
            None => Err(SomeAndError::IsNone(self.field)),
            Some(some) => self.inner.validate(some).map_err(SomeAndError::Some),
        }
    }
}