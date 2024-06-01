#[derive(Debug)]
struct SignupDataValidationErrors {
    mail: Option<::validator::InvalidEmailError>,
    site: Option<::validator::InvalidUrlError>,
    first_name: Option<::validator::InvalidLengthError>,
}

struct SignupDataValidator {
    mail: ::validator::EmailValidator,
    site: ::validator::UrlValidator,
    first_name: ::validator::LengthValidator,
}

impl ::validator::Validator<SignupData> for SignupDataValidator {
    type Error = SignupDataValidationErrors;
    fn validate(&self: Self, SignupData { mail, site, first_name }: SignupData) {
        let mut _valid = true;
        let validator = self;
        let error = SignupDataValidationErrors {
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
}

impl ::validator::Validate for SignupData {
    type Validator = SignupDataValidator;
    fn validator() -> Self::Validator {
        SignupDataValidator {
            mail: ::validator::EmailValidator,
            site: ::validator::UrlValidator,
            first_name: ::validator::LengthValidator::new(Some(1usize), None)
        }
    }
}