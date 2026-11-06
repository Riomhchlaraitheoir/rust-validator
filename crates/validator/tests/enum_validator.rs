#![allow(dead_code)]

use std::assert_matches;
use validator::Validate;
use validator::Validator;

fn main() {
    let request = Request::Signup {
        mail: "foo@example.com".to_string(),
        site: "http://example.com".to_string(),
        first_name: "Tim?".to_owned(),
    };
    let result = request.validate();
    assert_matches!(result, Ok(()));

    let request = Request::Signup {
        mail: "foo".to_string(),
        site: "http://example.com".to_string(),
        first_name: "".to_owned(),
    };
    let result = request.validate();
    let err = result.unwrap_err();
    let RequestValidationErrors::Signup { mail: Some(mail), site: None, first_name: Some(first_name) } = err else {
        panic!("unexpected error variant: {:?}", err)
    };

    assert_eq!("first_name length is 0, should be more than 1", mail.to_string());
    assert_eq!("first_name length is 0, should be more than 1", first_name.to_string());
}

#[derive(Validator)]
enum Request {
    Signup {
        #[validator(email)]
        mail: String,
        #[validator(url)]
        site: String,
        #[validator(length(1..))]
        first_name: String,
    },
    Login(#[validator(email)] String, #[validator(length(8..=64))] String),
    Logout
}