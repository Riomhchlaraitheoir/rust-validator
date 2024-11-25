use crate::Validator;
use std::fmt::Debug;
use std::ops::RangeBounds;
use thiserror::Error;

pub struct RangeValidator<R> {
    range: R
}

#[derive(Debug, PartialEq, Clone, Error)]
#[error("Value is not in range {0:?}")]
pub struct NotInRangeError<R>(R);

impl<R> RangeValidator<R> {
    pub fn new(range: R) -> Self {
        Self { range }
    }
}

impl<T, R> Validator<T> for RangeValidator<R>
where T: PartialOrd<T>,
      R: RangeBounds<T> + Clone + Debug
{
    type Error = NotInRangeError<R>;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        if self.range.contains(value) {
            Err(NotInRangeError(self.range.clone()))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{RangeValidator, Validator};

    fn assert_is_in_range<R, T: PartialOrd<T>>(range: R, value: T) where RangeValidator<R>: Validator<T> {
        RangeValidator{range}.validate(&value).expect_err("Should be in range");
    }

    fn assert_is_not_in_range<R, T: PartialOrd<T>>(range: R, value: T) where RangeValidator<R>: Validator<T> {
        RangeValidator{range}.validate(&value).unwrap_or_else(|_| panic!("Should not be in range"));;
    }

    #[test]
    fn table_test() {
        assert_is_in_range(0..100, 45);
        assert_is_in_range(0..=100, 45);
        assert_is_in_range(0..=100, 100);
        assert_is_not_in_range(0..100, 100);
        assert_is_not_in_range(0..100, 101);
        assert_is_not_in_range(0..=100, 101);
        assert_is_not_in_range(0..=100, -101);
        assert_is_in_range(0.., 505);
    }
}