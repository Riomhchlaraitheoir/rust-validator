use thiserror::Error;
use crate::Validator;

#[derive(Debug, Default)]
pub struct Or<A, B>(A, B);

#[derive(Debug, Error, PartialEq, Clone)]
#[error("{0} and {1}")]
pub struct OrError<A, B>(A, B);

impl<A, B, T> Validator<T> for Or<A, B> where A: Validator<T>, B: Validator<T> {
    type Error = OrError<A::Error, B::Error>;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        let Err(left) = self.0.validate(value) else { return Ok(()) };
        let Err(right) = self.1.validate(value) else { return Ok(()) };
        Err(OrError(left, right))
    }
}

impl<A, B> Or<A, B> {
    pub fn new(left: A, right: B) -> Self {
        Self(left, right)
    }
}
