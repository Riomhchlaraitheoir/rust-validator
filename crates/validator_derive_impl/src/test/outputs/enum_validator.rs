#[derive(Debug, PartialEq, Clone)]
enum RequestValidationErrors {
    Signup {
        mail: Option<::validator::InvalidEmailError>,
        site: Option<::validator::UrlValidationError>,
        first_name: Option<::validator::InvalidLengthError>,
    },
    Login(
        Option<::validator::InvalidEmailError>,
        Option<::validator::InvalidLengthError>,
    ),
}
#[doc(hidden)]
struct RequestSignupValidator {
    mail: ::validator::EmailValidator,
    site: ::validator::UrlValidator,
    first_name: ::validator::LengthValidator,
}
#[doc(hidden)]
struct RequestLoginValidator(::validator::EmailValidator, ::validator::LengthValidator);

struct RequestValidator(RequestSignupValidator, RequestLoginValidator);

#[automatically_derived]
impl ::validator::Validator<Request> for RequestValidator {
    type Error = RequestValidationErrors;
    #[allow(non_shorthand_field_patterns)]
    fn validate(&self, value: &Request) -> Result<(), Self::Error> {
        match value {
            Request::Signup { mail: mail, site: site, first_name: first_name } => {
                let mut _valid = true;
                let validator = &self.0;
                let error = RequestValidationErrors::Signup {
                    mail: {
                        match validator.mail.validate(mail) {
                            Ok(()) => None,
                            Err(error) => {
                                _valid = false;
                                Some(error)
                            }
                        }
                    },
                    site: {
                        match validator.site.validate(site) {
                            Ok(()) => None,
                            Err(error) => {
                                _valid = false;
                                Some(error)
                            }
                        }
                    },
                    first_name: {
                        match validator.first_name.validate(first_name) {
                            Ok(()) => None,
                            Err(error) => {
                                _valid = false;
                                Some(error)
                            }
                        }
                    },
                };
                if _valid { Ok(()) } else { Err(error) }
            }
            Request::Login(value0, value1) => {
                let mut _valid = true;
                let validator = &self.1;
                let error = RequestValidationErrors::Login {
                    0: {
                        match validator.0.validate(value0) {
                            Ok(()) => None,
                            Err(error) => {
                                _valid = false;
                                Some(error)
                            }
                        }
                    },
                    1: {
                        match validator.1.validate(value1) {
                            Ok(()) => None,
                            Err(error) => {
                                _valid = false;
                                Some(error)
                            }
                        }
                    },
                };
                if _valid { Ok(()) } else { Err(error) }
            }
            Request::Logout => Ok(()),
        }
    }
}
#[automatically_derived]
impl ::validator::Validate for Request {
    type Validator = RequestValidator;
    fn validator() -> Self::Validator {
        RequestValidator {
            0: RequestSignupValidator {
                mail: ::validator::EmailValidator::new("mail"),
                site: ::validator::UrlValidator::new("site"),
                first_name: ::validator::LengthValidator::new("first_name", 1..),
            },
            1: RequestLoginValidator {
                0: ::validator::EmailValidator::new("0"),
                1: ::validator::LengthValidator::new("1", 8..=64),
            },
        }
    }
}
