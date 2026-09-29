use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;
use thiserror::Error;
use crate::Validator;

#[derive(Clone, Error)]
#[error("Failed to parse {field}: {inner}")]
pub struct ParseAsError<T: FromStr<Err: Debug + Display>> {
    inner: T::Err,
    field: &'static str,
}

impl<T: FromStr<Err: Debug + Display + PartialEq>> PartialEq for ParseAsError<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner) && self.field.eq(other.field)
    }
}

impl<T: FromStr<Err: Debug + Display>> Debug for ParseAsError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ParseAsError")
        .field(&self.inner)
        .finish()
    }
}

pub struct ParseAs<T> {
    _t: PhantomData<T>,
    field: &'static str,
}

impl<T: FromStr<Err: Debug + Display>> ParseAs<T> {
    pub fn new(field: &'static str) -> Self {
        Self { _t: PhantomData, field }
    }
}

impl<T: FromStr<Err: Debug + Display>> Validator<str> for ParseAs<T> {
    type Error = ParseAsError<T>;

    fn validate(&self, value: &str) -> Result<(), Self::Error> {
        let Err(error) = T::from_str(value) else {
            return Ok(())
        };
        Err(ParseAsError {
            inner: error,
            field: self.field
        })
    }
}