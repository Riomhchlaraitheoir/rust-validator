use std::net::{AddrParseError, IpAddr};
use crate::Validator;

pub struct IpAddressValidator;

impl Validator<str> for IpAddressValidator {
    type Error = AddrParseError;

    fn validate(&self, value: &str) -> Result<(), Self::Error> {
        value.parse().map(|_: IpAddr| ())
    }
}