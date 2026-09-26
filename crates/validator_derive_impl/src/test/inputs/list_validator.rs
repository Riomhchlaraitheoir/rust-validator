
struct HasList {
    #[validator(elements)]
    list: Vec<Element>
}