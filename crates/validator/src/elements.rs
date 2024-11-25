use std::ops::Index;

use crate::{Validate, Validator};

pub struct ElementsValidator<V>(V);

#[derive(Debug, PartialEq, Clone)]
pub struct ElementsInvalid<E> {
    errors: Vec<Option<E>>
}

pub trait HasElements {
    type Item;
    fn _iter(&self) -> impl Iterator<Item=&Self::Item>;
}

impl<E> Index<usize> for ElementsInvalid<E> {
    type Output = Option<E>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.errors[index]
    }
}

impl<E, V> Validator<E> for ElementsValidator<V>
where
    E: HasElements,
    V: Validator<<E as HasElements>::Item>
{
    type Error = ElementsInvalid<V::Error>;

    fn validate(&self, slice: &E) -> Result<(), Self::Error> {
        let errors: Vec<_> = slice._iter().map(|element| {
            self.0.validate(element).err()
        }).collect();
        if errors.iter().all(Option::is_none) {
            Ok(())
        } else {
            Err(ElementsInvalid { errors })
        }
    }
}

impl<V> ElementsValidator<V> {
    pub fn new(validator: V) -> Self {
        Self(validator)
    }
}

impl<T> HasElements for [T] {
    type Item = T;

    fn _iter(&self) -> impl Iterator<Item=&Self::Item> {
        self.iter()
    }
}

impl<T> HasElements for Vec<T> {
    type Item = T;

    fn _iter(&self) -> impl Iterator<Item=&Self::Item> {
        self.iter()
    }
}

impl<T, const N: usize> HasElements for [T; N] {
    type Item = T;

    fn _iter(&self) -> impl Iterator<Item=&Self::Item> {
        self.iter()
    }
}

impl<E> Validate for E
where
    E: HasElements,
    <E as HasElements>::Item: Validate
{
    type Validator = ElementsValidator<<<E as HasElements>::Item as Validate>::Validator>;

    fn validator() -> Self::Validator {
        ElementsValidator(<E as HasElements>::Item::validator())
    }
}
