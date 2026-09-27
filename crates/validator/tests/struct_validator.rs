use validator::{AndError, Validate, Validator};

// include!("../../validator_derive_impl/src/test/outputs/struct_validator.rs");
#[derive(Validator)]
struct SignupData {
    #[validator(email)]
    mail: String,
    #[validator(url)]
    site: String,
    #[validator(not_empty)]
    first_name: String,
    #[validator(range(18..))]
    age: u8,
    #[validator(and(elements, length(min = 1)))]
    dogs: Vec<Dog>
}

#[derive(Validator)]
struct Dog {
    #[validator(length(max = 10))]
    name: String,
}

#[test]
fn struct_validation() {
    let data = SignupData {
        mail: "someone@example.com".to_string(),
        site: "http://example.com".to_string(),
        first_name: "Tom".to_string(),
        age: 42,
        dogs: vec![
            Dog {
                name: "Spot".to_string()
            }
        ],
    };
    let result: Result<(), SignupDataValidationErrors> = data.validate();
    result.expect("expected value to pass validation");
    let data = SignupData {
        mail: "someoneexample.com".to_string(),
        site: "httpexample.com".to_string(),
        first_name: "".to_string(),
        age: 12,
        dogs: vec![],
    };
    let result: Result<(), SignupDataValidationErrors> = data.validate();
    let error = result.expect_err("expected value to fail validation");
    assert_eq!(error.mail.unwrap().to_string(), "mail is not a valid email (No '@' character was found in the given address)");
    assert_eq!(error.site.unwrap().to_string(), "site is not a valid URL (relative URL without a base)");
    assert_eq!(error.first_name.unwrap().to_string(), "first_name should not be empty");
    assert_eq!(error.age.unwrap().to_string(), "age is not in range, must be greater than or equal to 18");
    let dogs_error = error.dogs;
    let AndError::Right(length_error) = dogs_error.clone().unwrap() else {
        panic!("Expected AndError::Right, got {:?}", dogs_error);
    };
    assert_eq!(length_error.to_string(), "dogs length is 0, should be more than 1");
}