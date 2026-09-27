use crate::ip::IpAddressValidator;
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use crate::{ErrorString, IpValidationError, ToErrors, Validator};

/// Validates whether the given string is an email based on the [HTML5 spec](https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address).
/// [RFC 5322](https://tools.ietf.org/html/rfc5322) is not practical in most circumstances and allows email addresses
/// that are unfamiliar to most users.
pub struct EmailValidator(&'static str);

impl EmailValidator {
    pub fn new(field: &'static str) -> EmailValidator {
        Self(field)
    }
}

// Regex from the specs
// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
// It will mark esoteric email addresses like quoted string as invalid
lazy_static!{
    static ref EMAIL_USER_RE: Regex = Regex::new(
        r"^\w([a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]?\w)*\z"
    ).unwrap();
    static ref EMAIL_DOMAIN_RE: Regex = Regex::new(
        r"^[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();
}

#[derive(Debug, Error, PartialEq, Clone)]
#[error("{field} is not a valid email ({reason})")]
pub struct InvalidEmailError {
    pub field: &'static str,
    pub reason: EmailInvalidReason,
}


#[derive(Debug, Error, PartialEq, Clone)]
pub enum EmailInvalidReason {
    #[error("An empty string is not a valid email")]
    EmptyValue,
    #[error("No '@' character was found in the given address")]
    ATNotFound,
    #[error("Multiple '@' characters were found in the given address")]
    MultipleATCharacters,
    #[error("The user part of the given address exceeds the maximum 64 characters")]
    UserPartTooLong,
    #[error("The domain part of the given address exceeds the maximum 255 characters")]
    DomainPartTooLong,
    #[error("The username part of the given address did not match the regex pattern")]
    InvalidUser,
    #[error("The domain part of the given address did not match the regex pattern")]
    InvalidDomain,
    #[error("The domain part of the given address was not a valid IP address: {0}")]
    InvalidAddr(IpValidationError)
}

impl ToErrors for InvalidEmailError {
    fn build_errors(&self, errors: &mut Vec<ErrorString>) {
        errors.push(self.to_string().into());
    }
}

impl Validator<str> for EmailValidator {
    type Error = InvalidEmailError;

    fn validate(&self, value: &str) -> Result<(), Self::Error> {
        self.validate_with_reason(value).map_err(|reason| {
            InvalidEmailError {
                field: self.0,
                reason,
            }
        })
    }
}
impl EmailValidator {
    fn validate_with_reason(&self, value: &str) -> Result<(), EmailInvalidReason> {
        if value.is_empty() {
            return Err(EmailInvalidReason::EmptyValue)
        }

        if value.chars().filter(|&c| c == '@').count() > 1 {
            return Err(EmailInvalidReason::MultipleATCharacters)
        }

        let (username, domain) = value.split_once('@')
            .ok_or(EmailInvalidReason::ATNotFound)?;

        if username.len() > 64 {
            return Err(EmailInvalidReason::UserPartTooLong)
        }
        if domain.len() > 255 {
            return Err(EmailInvalidReason::DomainPartTooLong)
        }

        if !EMAIL_USER_RE.is_match(username) {
            return Err(EmailInvalidReason::InvalidUser)
        }
        if !EMAIL_DOMAIN_RE.is_match(domain) {
            let ip = domain.strip_prefix('[').and_then(|domain|
            domain.strip_suffix(']'));
            return if let Some(ip) = ip {
                match IpAddressValidator::new(self.0).validate(ip) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(EmailInvalidReason::InvalidAddr(error))
                }
            } else {
                Err(EmailInvalidReason::InvalidDomain)
            }
        }
        Ok(())
    }
}