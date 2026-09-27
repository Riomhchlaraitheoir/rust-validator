#[derive(Validator)]
enum Request {
    Signup {
        #[validator(email)]
        mail: String,
        #[validator(url)]
        site: String,
        #[validator(length(min = 1))]
        first_name: String,
    },
    Login(#[validator(email)] String, #[validator(length(min = 8, max = 64))] String),
    Logout
}