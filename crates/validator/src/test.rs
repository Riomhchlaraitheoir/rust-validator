#![cfg(test)]

use crate::{EmailValidator, Validator};

trait ValidatorAssertion<T: ?Sized>: Validator<T> {
    fn assert_valid(&self, value: &T) {
        self.validate(value).expect("should be Ok")
    }

    fn assert_invalid(&self, value: &T) {
        self.validate(value).expect_err("should be Err");
    }

    fn assert_invalid_err(&self, value: &T, expected_error: Self::Error) where Self::Error: PartialEq {
        let actual_error = self.validate(value).expect_err("should be Err");
        assert_eq!(expected_error, actual_error)
    }

}

impl<T: ?Sized, V: Validator<T>> ValidatorAssertion<T> for V {}

#[test]
fn email() {
    let v = EmailValidator;
    v.assert_valid("email@example.com");
    v.assert_valid("firstname.lastname@example.com");
    v.assert_valid("email@subdomain.example.com");
    v.assert_valid("firstname+lastname@example.com");
    v.assert_valid("email@123.123.123.123");
    v.assert_valid("email@[123.123.123.123]");
    v.assert_valid("1234567890@example.com");
    v.assert_valid("email@example-one.com");
    v.assert_valid("_______@example.com");
    v.assert_valid("email@example.name");
    v.assert_valid("email@example.museum");
    v.assert_valid("email@example.co.jp");
    v.assert_valid("firstname-lastname@example.com");

    v.assert_invalid(r#"plainaddress"#);
    v.assert_invalid(r#"#@%^%#$@#$@#.com"#);
    v.assert_invalid(r#"@example.com"#);
    v.assert_invalid(r#"Joe Smith <email@example.com>"#);
    v.assert_invalid(r#"email.example.com"#);
    v.assert_invalid(r#"email@example@example.com"#);
    v.assert_invalid(r#".email@example.com"#);
    v.assert_invalid(r#"email.@example.com"#);
    v.assert_invalid(r#"email..email@example.com"#);
    v.assert_invalid(r#"email@example.com (Joe Smith)"#);
    v.assert_invalid(r#"email@-example.com"#);
    v.assert_invalid(r#"email@example..com"#);
    v.assert_invalid(r#"Abc..123@example.com"#);
    // unusual examples
    v.assert_invalid(r#"”(),:;<>[\]@example.com"#);
    v.assert_invalid(r#"just”not”right@example.com"#);
    v.assert_invalid(r#"this\ is"really"not\allowed@example.com"#);
}