use std::{fmt::Write, marker::PhantomData};

use crate::sql::{
    ParameterBinder, SelectableTables, SqlAdd, SqlDivide, SqlExpression, SqlMultiply, SqlSubtract,
};

macro_rules! define_operator{
    {$type_name: ident, $sql_type_marker: ident, $operator: tt} => {
        pub struct $type_name<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S>>
        where
            Lhs::SqlType: $sql_type_marker<Rhs::SqlType>
        {
            lhs: Lhs,
            rhs: Rhs,
            phantom: PhantomData<S>,
        }

        impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S>>
            $type_name<S, Lhs, Rhs>
        where
            Lhs::SqlType: $sql_type_marker<Rhs::SqlType>
        {
            pub fn new(lhs: Lhs, rhs: Rhs) -> Self {
                Self {
                    lhs,
                    rhs,
                    phantom: PhantomData,
                }
            }
        }

        impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S>> SqlExpression<S>
            for $type_name<S, Lhs, Rhs>
        where
            Lhs::SqlType: $sql_type_marker<Rhs::SqlType>
        {
            type SqlType = Lhs::SqlType;
            type RustType = Lhs::RustType;

            const IS_AGGREGATE:bool = Lhs::IS_AGGREGATE || Rhs::IS_AGGREGATE;

            fn write_sql_string<'s, 'a>(
                &'s self,
                f: &mut String,
                parameter_binder: &mut ParameterBinder<'a>,
            ) -> std::fmt::Result
            where
                's: 'a,
            {
                self.lhs.write_parenthesized_sql_string(f, parameter_binder)?;
                write!(f, stringify!($operator))?;
                self.rhs.write_parenthesized_sql_string(f, parameter_binder)?;
                Ok(())
            }
        }

    }
}

define_operator! {SqlAddition, SqlAdd, +}
define_operator! {SqlSubtraction, SqlSubtract, -}
define_operator! {SqlMultiplication, SqlMultiply, *}
define_operator! {SqlDivision, SqlDivide, /}
