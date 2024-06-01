macro_rules! tuple {
    (($t:ident), ($v:ident), ($e:ident), ($i:tt)) => {};
    (($v_head:ident $(,$v:ident)*), ($t_head:ident $(,$t:ident)*), ($e_head:ident $(,$e:ident)*), ($i_head:tt $(,$i:tt)*)) => {
        tuple!(($($v),*), ($($t),*), ($($e),*), ($($i),*));
        tuple!(impl ($v_head $(,$v)*), ($t_head $(,$t)*), $e_head, ($i_head $(,$i)*));
    };
    (impl ($($v:ident),*), ($($t:ident),*), $e:ident, ($($i:tt),*)) => {

#[derive(Debug, PartialEq, Clone)]
pub struct $e< $($t),* >($(pub Option<$t>),*);

impl<$($v: crate::Validator<$t>, $t),*> crate::Validator<( $($t),* )> for ($($v),*) {
    type Error = $e< $($v::Error),* >;

    fn validate(&self, value: &( $($t),* )) -> Result<(), Self::Error> {
        let mut valid = true;
        let error = $e {
            $($i: {
                match self.$i.validate(&value.$i) {
                    Ok(()) => None,
                    Err(error) => {
                        valid = false;
                        Some(error)
                    }
                }
            }),*
        };

        if valid {
            Ok(())
        } else {
            Err(error)
        }
    }
}

    };
}

tuple!((V12, V11, V10, V9, V8, V7, V6, V5, V4, V3, V2, V1, V0), (T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1, T0), (TupleError13, TupleError12, TupleError11, TupleError10, TupleError9, TupleError8, TupleError7, TupleError6, TupleError5, TupleError4, TupleError3, TupleError2, TupleError1), (12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0));
