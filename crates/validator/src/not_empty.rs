use crate::length::HasLength;
use crate::{ErrorString, ToErrors, Validator};
use thiserror::Error;

pub struct NotEmptyValidator(&'static str);

impl NotEmptyValidator {
    pub fn new(field: &'static str) -> NotEmptyValidator {
        Self(field)
    }
}

#[derive(Debug, PartialEq, Clone, Error)]
#[error("{0} should not be empty")]
pub struct EmptyValueError(&'static str);

impl ToErrors for EmptyValueError {
    fn build_errors(&self, errors: &mut Vec<ErrorString>) {
        errors.push(self.to_string().into());
    }
}

impl<T: ?Sized> Validator<T> for NotEmptyValidator where T: HasLength {
    type Error = EmptyValueError;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        if value._len() == 0 {
            Err(EmptyValueError(self.0))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::not_empty::NotEmptyValidator;
    use crate::Validator;

    fn assert_is_empty<T: ?Sized>(value: &T) where NotEmptyValidator: Validator<T> {
        NotEmptyValidator("test").validate(value).expect_err("Should be empty");
    }

    fn assert_not_empty<T: ?Sized>(value: &T) where NotEmptyValidator: Validator<T> {
        NotEmptyValidator("test").validate(value).unwrap_or_else(|_| panic!("Should not be empty"));
    }

    #[test]
    fn table_test() {
        assert_is_empty("");
        assert_is_empty(&[0_u8; 0]);
        assert_is_empty(&[] as &[&str]);
        assert_is_empty(&String::default());
        assert_is_empty(&String::with_capacity(5));
        assert_is_empty(&Vec::<u8>::with_capacity(5));

        assert_not_empty("foo");
        assert_not_empty(&[1]);
        assert_not_empty(&String::from("foo"));
        assert_not_empty(&vec![1, 2, 3])
    }
}