#![allow(dead_code)]

use validator_derive::Validator;

fn main() {

}

#[derive(Validator)]
struct SignupForm {
    #[validator(and(not_empty, length(..=20)))]
    password: String,
    #[validator(matches(password))]
    confirm_password: String,
}