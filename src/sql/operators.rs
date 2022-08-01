use std::{fmt::Write, marker::PhantomData};

use super::{SqlAdd, SqlDivide, SqlMultiply, SqlSubtract, SqlI64};
use crate::sql::{ParameterBinder, SelectableTables, SqlExpression};

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

            fn write_sql_string<'s, 'a>(
                &'s self,
                f: &mut String,
                parameter_binder: &mut ParameterBinder<'a>,
            ) -> std::fmt::Result
            where
                's: 'a,
            {
                self.lhs.write_sql_string(f, parameter_binder)?;
                write!(f, stringify!($operator))?;
                self.rhs.write_sql_string(f, parameter_binder)?;
                Ok(())
            }
        }
    }
}

define_operator! {SqlAddition, SqlAdd, +}
define_operator! {SqlSubtraction, SqlSubtract, -}
define_operator! {SqlMultiplication, SqlMultiply, *}
define_operator! {SqlDivision, SqlDivide, /}

pub struct SqlCount<S: SelectableTables, E: SqlExpression<S>> {
    expr: E,
    phantom: PhantomData<S>,
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlCount<S, E> {
    pub fn new(expr: E) -> Self {
        Self {
            expr,
            phantom: PhantomData,
        }
    }
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlExpression<S>
    for SqlCount<S, E>
{
    type SqlType = SqlI64;
    type RustType = i64;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "COUNT(")?;
        self.expr.write_sql_string(f, parameter_binder)?;
        write!(f, ")")?;
        Ok(())
    }
}
