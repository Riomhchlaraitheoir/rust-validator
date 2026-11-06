#![allow(dead_code)]

use validator_derive::Validator;

fn main() {

}

#[derive(Validator)]
struct Element {
    #[validator(not_empty)]
    name: String
}

#[derive(Validator)]
struct HasList {
    #[validator(elements)]
    list: Vec<Element>
}