use crate::{ErrorString, ToErrors, Validator};
use std::collections::Bound;
use std::fmt::Debug;
use std::ops::RangeBounds;
use thiserror::Error;

pub struct RangeValidator<R> {
    field: &'static str,
    range: R
}

#[derive(Debug, PartialEq, Clone, Error)]
#[error("{0} is not in range, must be {2}")]
pub struct NotInRangeError<R>(&'static str, R, String);

impl<R> RangeValidator<R> {
    pub fn new(field: &'static str, range: R) -> Self {
        Self { field, range }
    }
}

impl<R: Debug + Clone + 'static> ToErrors for NotInRangeError<R> {
    fn build_errors(&self, errors: &mut Vec<ErrorString>) {
        errors.push(self.to_string().into());
    }
}

impl<T, R> Validator<T> for RangeValidator<R>
where T: PartialOrd<T> + Debug,
      R: RangeBounds<T> + Clone + Debug
{
    type Error = NotInRangeError<R>;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        if self.range.contains(value) {
            Ok(())
        } else {
            Err(NotInRangeError(self.field, self.range.clone(), range_desc(&self.range)))
        }
    }
}

fn range_desc<R: RangeBounds<T>, T: Debug>(range: &R) -> String {
    let start = match range.start_bound() {
        Bound::Included(value) => Some(format!("greater than or equal to {value:?}")),
        Bound::Excluded(value) => Some(format!("greater than {value:?}")),
        Bound::Unbounded => None
    };
    let end = match range.end_bound() {
        Bound::Included(value) => Some(format!("less than or equal to {value:?}")),
        Bound::Excluded(value) => Some(format!("less than {value:?}")),
        Bound::Unbounded => None
    };
    [start, end].into_iter().flatten().collect::<Vec<_>>().join(" and ")
}


#[cfg(test)]
mod test {
    use crate::{RangeValidator, Validator};

    fn assert_is_in_range<R, T: PartialOrd<T>>(range: R, value: T) where RangeValidator<R>: Validator<T> {
        RangeValidator{field: "test", range}.validate(&value).expect("Should be in range");
    }

    fn assert_is_not_in_range<R, T: PartialOrd<T>>(range: R, value: T) where RangeValidator<R>: Validator<T> {
        RangeValidator{field: "test", range}.validate(&value).expect_err("Should not be in range");
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