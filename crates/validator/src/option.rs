use crate::{Validator};

/// Validates the value if it is present
pub struct OptionValidator<V> {
    inner: V,
}

impl<V> OptionValidator<V> {
    pub fn new(inner: V) -> Self {
        OptionValidator { inner }
    }
}

impl<V: Validator<T>, T> Validator<Option<T>> for OptionValidator<V> {
    type Error = V::Error;
    fn validate(&self, value: &Option<T>) -> Result<(), Self::Error> {
        if let Some(value) = value {
            self.inner.validate(value)
        } else {
            Ok(())
        }
    }
}

pub trait OptionValue {
    type Inner;
}

impl<T> OptionValue for Option<T> {
    type Inner = T;
}