use crate::Validator;

pub struct UrlValidator;
pub use url::ParseError as InvalidUrlError;

impl Validator<str> for UrlValidator {
    type Error = InvalidUrlError;

    fn validate(&self, value: &str) -> Result<(), Self::Error> {
        url::Url::parse(value).map(|_| ())
    }
}