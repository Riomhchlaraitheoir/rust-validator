#[derive(Validator)]
struct HasList {
    #[validator(elements)]
    list: Vec<Element>
}