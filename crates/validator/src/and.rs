use thiserror::Error;
use crate::Validator;

#[derive(Debug, Default, PartialEq)]
pub struct And<A, B>(A, B);

#[derive(Debug, Error, PartialEq, Clone)]
pub enum AndError<A, B> {
    #[error(transparent)]
    Left(A),
    #[error(transparent)]
    Right(B),
    #[error("{0} and {1}")]
    Both(A, B)
}

impl<A, B> From<AndError<A, B>> for (Option<A>, Option<B>) {
    fn from(value: AndError<A, B>) -> Self {
        match value {
            AndError::Left(left) => (Some(left), None),
            AndError::Right(right) => (None, Some(right)),
            AndError::Both(left, right) => (Some(left), Some(right)),
        }
    }
}

impl<A, B, T> Validator<T> for And<A, B> where A: Validator<T>, B: Validator<T> {
    type Error = AndError<A::Error, B::Error>;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        match (self.0.validate(value), self.1.validate(value)) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(left), Ok(())) => Err(AndError::Left(left)),
            (Ok(()), Err(right)) => Err(AndError::Right(right)),
            (Err(left), Err(right)) => Err(AndError::Both(left, right))
        }
    }
}

impl<A, B> And<A, B> {
    pub fn new(left: A, right: B) -> Self {
        Self(left, right)
    }
}
