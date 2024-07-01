use validator::{Validate, Validator};

#[derive(Validator)]
struct HasList {
    #[validator(elements)]
    list: Vec<Element>
}

#[derive(Validator)]
struct Element {
    #[validator(not_empty)]
    value: String
}

#[test]
fn test() {
    HasList {
        list: vec![
            Element {
                value: "foobar".to_string(),
            }
        ],
    }.validate().expect("should be valid");
    HasList {
        list: vec![
            Element {
                value: "".to_string(),
            }
        ],
    }.validate().expect_err("should be invalid");
}