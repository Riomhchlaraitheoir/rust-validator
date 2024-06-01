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

impl<T, V: Validator<T>> Validator<[T]> for ElementsValidator<V> {
    type Error = ElementsInvalid<V::Error>;

    fn validate(&self, slice: &[T]) -> Result<(), Self::Error> {
        let errors: Vec<_> = slice.iter().map(|element| {
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

impl<T: Validate> Validate for [T] {
    type Validator = ElementsValidator<T::Validator>;

    fn validator() -> Self::Validator {
        ElementsValidator(T::validator())
    }
}

impl<T: Validate> Validate for Vec<T> {
    type Validator = ElementsValidator<T::Validator>;

    fn validator() -> Self::Validator {
        ElementsValidator(T::validator())
    }
}

impl<T: Validate, const N: usize> Validate for [T; N] {
    type Validator = ElementsValidator<T::Validator>;

    fn validator() -> Self::Validator {
        ElementsValidator(T::validator())
    }
}
