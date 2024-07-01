use std::ops::Index;

use crate::{Validate, Validator};

pub struct ElementsValidator<V>(V);

#[derive(Debug, PartialEq, Clone)]
pub struct ElementsInvalid<E> {
    errors: Vec<Option<E>>
}

impl<E> Index<usize> for ElementsInvalid<E> {
    type Output = Option<E>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.errors[index]
    }
}

impl<T, V: Validator<T>, I: HasElements<Element = T>> Validator<I> for ElementsValidator<V> {
    type Error = ElementsInvalid<V::Error>;

    fn validate(&self, slice: &I) -> Result<(), Self::Error> {
        let errors: Vec<_> = slice.iterator().map(|element| {
            self.0.validate(element).err()
        }).collect();
        if errors.iterator().all(Option::is_none) {
            Ok(())
        } else {
            Err(ElementsInvalid { errors })
        }
    }
}

pub trait HasElements {
    type Element: Sized;

    fn iterator(&self) -> impl Iterator<Item = &Self::Element>;
}

impl<T> HasElements for [T] {
    type Element = T;

    fn iterator(&self) -> impl Iterator<Item=&Self::Element> {
        self.iter()
    }
}

impl<T, const N: usize> HasElements for [T; N] {
    type Element = T;

    fn iterator(&self) -> impl Iterator<Item=&Self::Element> {
        self.iter()
    }
}

impl<T> HasElements for Vec<T> {
    type Element = T;

    fn iterator(&self) -> impl Iterator<Item=&Self::Element> {
        self.iter()
    }
}

impl<V> ElementsValidator<V> {
    pub fn new(validator: V) -> Self {
        Self(validator)
    }
}

impl<T: Validate, I: HasElements<Element = T>> Validate for I {
    type Validator = ElementsValidator<T::Validator>;

    fn validator() -> Self::Validator {
        ElementsValidator(T::validator())
    }
}

// impl<T: Validate> Validate for [T] {
//     type Validator = ElementsValidator<T::Validator>;
//
//     fn validator() -> Self::Validator {
//         ElementsValidator(T::validator())
//     }
// }
//
// impl<T: Validate> Validate for Vec<T> {
//     type Validator = ElementsValidator<T::Validator>;
//
//     fn validator() -> Self::Validator {
//         ElementsValidator(T::validator())
//     }
// }
//
// impl<T: Validate, const N: usize> Validate for [T; N] {
//     type Validator = ElementsValidator<T::Validator>;
//
//     fn validator() -> Self::Validator {
//         ElementsValidator(T::validator())
//     }
// }
