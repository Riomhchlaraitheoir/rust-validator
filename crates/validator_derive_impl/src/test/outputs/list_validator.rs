#[derive(Debug, PartialEq, Clone, Default)]
struct HasListValidationErrors {
    list: Option<
        ::validator::ElementsInvalid<
            <<<Vec<
                Element,
            > as ::validator::HasElements>::Item as ::validator::Validate>::Validator as ::validator::Validator<
                <Vec<Element> as ::validator::HasElements>::Item,
            >>::Error,
        >,
    >,
}
struct HasListValidator {
    list: ::validator::ElementsValidator<
        <<Vec<
            Element,
        > as ::validator::HasElements>::Item as ::validator::Validate>::Validator,
    >,
}
impl ::validator::Validator<HasList> for HasListValidator {
    type Error = HasListValidationErrors;
    #[allow(non_shorthand_field_patterns)]
    fn validate(&self, HasList { list: list }: &HasList) -> Result<(), Self::Error> {
        let mut _valid = true;
        let validator = self;
        let error = HasListValidationErrors {
            list: {
                match validator.list.validate(list) {
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
impl ::validator::Validate for HasList {
    type Validator = HasListValidator;
    fn validator() -> Self::Validator {
        HasListValidator {
            list: ::validator::ElementsValidator::new(
                <<Vec<
                    Element,
                > as ::validator::HasElements>::Item as ::validator::Validate>::validator(),
            ),
        }
    }
}
