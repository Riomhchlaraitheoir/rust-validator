use thiserror::Error;
use crate::length::HasLength;
use crate::Validator;

pub struct NotEmptyValidator;

#[derive(Debug, PartialEq, Clone, Error)]
#[error("Value should not be empty")]
pub struct EmptyValueError;

impl<T: ?Sized> Validator<T> for NotEmptyValidator where T: HasLength {
    type Error = EmptyValueError;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        if value._len() == 0 {
            Err(EmptyValueError)
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
        NotEmptyValidator.validate(value).expect_err("Should be empty");
    }

    fn assert_not_empty<T: ?Sized>(value: &T) where NotEmptyValidator: Validator<T> {
        NotEmptyValidator.validate(value).unwrap_or_else(|_| panic!("Should not be empty"));
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