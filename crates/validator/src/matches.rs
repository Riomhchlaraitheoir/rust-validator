use thiserror::Error;
use crate::{ErrorString, ToErrors, Validator};

pub struct Matches {
    field: &'static str,
    other: &'static str
}

impl Matches {
    pub fn new(field: &'static str, other: &'static str) -> Self {
        Self { field, other }
    }
}

impl<T: PartialEq> Validator<(&T, &T)> for Matches {
    type Error = MatchesError;

    fn validate(&self, (a, b): &(&T, &T)) -> Result<(), Self::Error> {
        if a == b {
            Ok(())
        } else {
            Err(MatchesError { field: self.field, other: self.other })
        }
    }
}

#[derive(Debug, PartialEq, Clone, Error)]
#[error("{field} does not match {other}")]
pub struct MatchesError{
    field: &'static str,
    other: &'static str
}

impl ToErrors for MatchesError {
    fn build_errors(&self, errors: &mut Vec<ErrorString>) {
        errors.push(self.to_string().into())
    }
}
