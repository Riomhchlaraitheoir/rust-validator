use thiserror::Error;
use crate::Validator;

pub struct LengthValidator(Option<usize>, Option<usize>);

#[derive(Debug, Error, PartialEq, Clone)]
pub enum InvalidLengthError {
    #[error("value of length {len} exceeds maximum of {max}")]
    TooLong {
        max: usize,
        len: usize
    },
    #[error("value of length {len} falls short of minimum of {min}")]
    TooShort {
        min: usize,
        len: usize
    }
}

pub(crate) trait HasLength {
    fn _len(&self) -> usize;
}

impl LengthValidator {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self(min, max)
    }
}

impl<T: HasLength + ?Sized> Validator<T> for LengthValidator {
    type Error = InvalidLengthError;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        let len = value._len();
        let Self(min, max) = self;
        if let Some(&min) = min.as_ref() {
            if len < min {
                return Err(InvalidLengthError::TooShort { min, len })
            }
        }
        if let Some(&max) = max.as_ref() {
            if len > max {
                return Err(InvalidLengthError::TooLong { max, len })
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