#[derive(Debug, PartialEq, Clone, Default)]
struct SignupDataValidationErrors {
    mail: Option<::validator::InvalidEmailError>,
    site: Option<::validator::InvalidUrlError>,
    first_name: Option<::validator::InvalidLengthError>,
    age: Option<::validator::NotInRangeError<::std::ops::RangeFrom<u8>>>,
    dogs: Option<
        ::validator::AndError<
            ::validator::ElementsInvalid<
                <<<Vec<
                    Dog,
                > as ::validator::HasElements>::Item as ::validator::Validate>::Validator as ::validator::Validator<
                    <Vec<Dog> as ::validator::HasElements>::Item,
                >>::Error,
            >,
            ::validator::InvalidLengthError,
        >,
    >,
}
struct SignupDataValidator {
    mail: ::validator::EmailValidator,
    site: ::validator::UrlValidator,
    first_name: ::validator::LengthValidator,
    age: ::validator::RangeValidator<::std::ops::RangeFrom<u8>>,
    dogs: ::validator::And<
        ::validator::ElementsValidator<
            <<Vec<
                Dog,
            > as ::validator::HasElements>::Item as ::validator::Validate>::Validator,
        >,
        ::validator::LengthValidator,
    >,
}
impl ::validator::Validator<SignupData> for SignupDataValidator {
    type Error = SignupDataValidationErrors;
    fn validate(
        &self,
        SignupData { mail, site, first_name, age, dogs }: &SignupData,
    ) -> Result<(), Self::Error> {
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
            age: {
                match validator.age.validate(age) {
                    Ok(()) => None,
                    Err(error) => {
                        _valid = false;
                        Some(error)
                    }
                }
            },
            dogs: {
                match validator.dogs.validate(dogs) {
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
            first_name: ::validator::LengthValidator::new(Some(1usize), None),
            age: ::validator::RangeValidator::new(18..),
            dogs: ::validator::And::new(
                ::validator::ElementsValidator::new(
                    <<Vec<
                        Dog,
                    > as ::validator::HasElements>::Item as ::validator::Validate>::validator(),
                ),
                ::validator::LengthValidator::new(Some(1usize), None),
            ),
        }
    }
}
