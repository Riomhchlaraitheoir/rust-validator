
#[derive(Validator)]
struct SignupForm {
    #[validator(and(not_empty, length(max = 20)))]
    password: String,
    #[validator(matches(password))]
    confirm_password: String,
}