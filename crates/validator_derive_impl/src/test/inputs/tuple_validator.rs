
struct SignupData(
    #[validator(email)]
    String,
    #[validator(url)]
    String,
    #[validator(length(min = 1))]
    String,
);