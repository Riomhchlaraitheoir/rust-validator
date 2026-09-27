use crate::{ErrorString, ToErrors, Validator};
use thiserror::Error;

pub struct LengthValidator {
    field: &'static str,
    min: Option<usize>,
    max: Option<usize>,
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum InvalidLengthError {
    #[error("{field} length is {len}, should be less than {max}")]
    TooLong {
        field: &'static str,
        max: usize,
        len: usize
    },
    #[error("{field} length is {len}, should be more than {min}")]
    TooShort {
        field: &'static str,
        min: usize,
        len: usize
    }
}

impl ToErrors for InvalidLengthError {
    fn build_errors(&self, errors: &mut Vec<ErrorString>) {
        errors.push(self.to_string().into());
    }
}

pub(crate) trait HasLength {
    fn _len(&self) -> usize;
}

impl LengthValidator {
    pub fn new(field: &'static str, min: Option<usize>, max: Option<usize>) -> Self {
        Self { field, min, max }
    }
}

impl<T: HasLength + ?Sized> Validator<T> for LengthValidator {
    type Error = InvalidLengthError;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        let len = value._len();
        let Self { field, min, max } = self;
        if let Some(&min) = min.as_ref() {
            if len < min {
                return Err(InvalidLengthError::TooShort { field, min, len })
            }
        }
        if let Some(&max) = max.as_ref() {
            if len > max {
                return Err(InvalidLengthError::TooLong { field, max, len })
            }
        }
        Ok(())
    }
}

impl HasLength for str {
    fn _len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for [T] {
    fn _len(&self) -> usize {
        self.len()
    }
}