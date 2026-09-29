#[derive(Debug, PartialEq, Clone, Default)]
struct SignupDataValidationErrors {
    mail: Option<::validator::InvalidEmailError>,
    site: Option<::validator::UrlValidationError>,
    first_name: Option<::validator::EmptyValueError>,
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
    first_name: ::validator::NotEmptyValidator,
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
    #[allow(non_shorthand_field_patterns)]
    fn validate(
        &self,
        SignupData { mail: mail, site: site, first_name: first_name, age: age, dogs: dogs }: &SignupData,
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
            mail: ::validator::EmailValidator::new("mail"),
            site: ::validator::UrlValidator::new("site"),
            first_name: ::validator::NotEmptyValidator::new("first_name"),
            age: ::validator::RangeValidator::new("age", 18..),
            dogs: ::validator::And::new(
                ::validator::ElementsValidator::new(
                    <<Vec<
                        Dog,
                    > as ::validator::HasElements>::Item as ::validator::Validate>::validator(),
                ),
                ::validator::LengthValidator::new("dogs", Some(1usize), None),
            ),
        }
    }
}
#[derive(Debug, PartialEq, Clone, Default)]
struct DogValidationErrors {
    name: Option<::validator::EmptyValueError>,
    age: Option<::validator::ParseAsError<u32>>,
}
struct DogValidator {
    name: ::validator::NotEmptyValidator,
    age: ::validator::ParseAs<u32>
}
impl ::validator::Validator<Dog> for DogValidator {
    type Error = DogValidationErrors;
    #[allow(non_shorthand_field_patterns)]
    fn validate(&self, Dog { name: name, age: age }: &Dog) -> Result<(), Self::Error> {
        let mut _valid = true;
        let validator = self;
        let error = DogValidationErrors {
            name: {
                match validator.name.validate(name) {
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
        };
        if _valid { Ok(()) } else { Err(error) }
    }
}
impl ::validator::Validate for Dog {
    type Validator = DogValidator;
    fn validator() -> Self::Validator {
        DogValidator {
            name: ::validator::NotEmptyValidator::new("name"),
            age: ::validator::ParseAs::<u32>::new("age"),
        }
    }
}
