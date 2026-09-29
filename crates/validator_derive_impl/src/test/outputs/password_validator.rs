#[derive(Debug, PartialEq, Clone, Default)]
struct SignupFormValidationErrors {
    password: Option<
        ::validator::AndError<
            ::validator::EmptyValueError,
            ::validator::InvalidLengthError,
        >,
    >,
    confirm_password: Option<::validator::MatchesError>,
}
struct SignupFormValidator {
    password: ::validator::And<
        ::validator::NotEmptyValidator,
        ::validator::LengthValidator,
    >,
    confirm_password: ::validator::Matches,
}
impl ::validator::Validator<SignupForm> for SignupFormValidator {
    type Error = SignupFormValidationErrors;
    #[allow(non_shorthand_field_patterns)]
    fn validate(
        &self,
        SignupForm {
            password: password,
            confirm_password: confirm_password,
        }: &SignupForm,
    ) -> Result<(), Self::Error> {
        let mut _valid = true;
        let validator = self;
        let error = SignupFormValidationErrors {
            password: {
                match validator.password.validate(password) {
                    Ok(()) => None,
                    Err(error) => {
                        _valid = false;
                        Some(error)
                    }
                }
            },
            confirm_password: {
                match validator.confirm_password.validate(&(confirm_password, password)) {
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
impl ::validator::Validate for SignupForm {
    type Validator = SignupFormValidator;
    fn validator() -> Self::Validator {
        SignupFormValidator {
            password: ::validator::And::new(
                ::validator::NotEmptyValidator::new("password"),
                ::validator::LengthValidator::new("password", None, Some(20usize)),
            ),
            confirm_password: ::validator::Matches::new("confirm_password", "password"),
        }
    }
}
