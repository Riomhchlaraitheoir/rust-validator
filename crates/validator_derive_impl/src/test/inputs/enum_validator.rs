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