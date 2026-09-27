use validator::Validate;
use validator_derive::Validator;

#[derive(Validator)]
struct SignupForm {
    #[validator(and(not_empty, length(max = 20)))]
    password: String,
    #[validator(matches(password))]
    confirm_password: String,
}

#[test]
fn test_matches() {
    let data = SignupForm {
        password: "123456".to_string(),
        confirm_password: "123456".to_string(),
    };
    data.validate().expect("should pass validation");

    let data = SignupForm {
        password: "123456".to_string(),
        confirm_password: "7890".to_string(),
    };
    data.validate().expect_err("should fail validation");
}