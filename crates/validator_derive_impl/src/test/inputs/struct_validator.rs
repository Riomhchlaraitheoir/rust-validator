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
    #[validator(not_empty)]
    name: String,
}
