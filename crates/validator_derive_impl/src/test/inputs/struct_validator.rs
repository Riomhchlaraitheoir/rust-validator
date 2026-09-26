
struct SignupData {
    #[validator(email)]
    mail: String,
    #[validator(url)]
    site: String,
    #[validator(length(min = 1))]
    first_name: String,
    #[validator(range(18..))]
    age: u8,
    #[validator(and(elements, length(min = 1)))]
    dogs: Vec<Dog>
}