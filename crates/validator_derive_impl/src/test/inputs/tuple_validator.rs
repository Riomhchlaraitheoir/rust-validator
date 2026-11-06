#[derive(Validator)]
struct SignupData(
    #[validator(email)]
    String,
    #[validator(url)]
    String,
    #[validator(length(1..))]
    String,
);