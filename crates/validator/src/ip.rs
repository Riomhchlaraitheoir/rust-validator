use crate::{ErrorString, ToErrors, Validator};
use std::net::{AddrParseError, IpAddr};
use thiserror::Error;

pub struct IpAddressValidator(&'static str);

#[derive(Debug, Error, Clone, PartialEq)]
#[error("{0} is not a valid IP address: {1}")]
pub struct IpValidationError(&'static str, AddrParseError);

impl ToErrors for IpValidationError {
    fn build_errors(&self, errors: &mut Vec<ErrorString>) {
        errors.push(self.to_string().into());
    }
}

impl IpAddressValidator {
    pub fn new(field: &'static str) -> IpAddressValidator {
        IpAddressValidator(field)
    }
}

impl Validator<str> for IpAddressValidator {
    type Error = IpValidationError;

    fn validate(&self, value: &str) -> Result<(), Self::Error> {
        match value.parse() {
            Ok(ip) => {
                let _: IpAddr = ip;
                Ok(())
            },
            Err(error) => Err(IpValidationError(self.0, error))
        }
    }
}