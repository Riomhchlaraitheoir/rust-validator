use crate::{ErrorString, ToErrors, Validator};
use thiserror::Error;
use url::{ParseError, Url};

pub struct UrlValidator(&'static str);

impl UrlValidator {
    pub fn new(field: &'static str) -> UrlValidator {
        Self(field)
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
#[error("{0} is not a valid URL ({1})")]
pub struct UrlValidationError(&'static str, ParseError);

impl ToErrors for UrlValidationError {
    fn build_errors(&self, errors: &mut Vec<ErrorString>) {
        errors.push(self.to_string().into())
    }
}

impl Validator<str> for UrlValidator {
    type Error = UrlValidationError;

    fn validate(&self, value: &str) -> Result<(), Self::Error> {
        match value.parse() {
            Ok(ip) => {
                let _: Url = ip;
                Ok(())
            },
            Err(error) => Err(UrlValidationError(self.0, error))
        }
    }
}