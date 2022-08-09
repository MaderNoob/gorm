use std::{fmt::Write, marker::PhantomData};

use crate::sql::{ParameterBinder, SelectableTables, SqlBool, SqlExpression};

/// Defines a new boolean operator which accepts 2 operands.
macro_rules! define_boolean_binary_operator {
    {$type_name: ident, $operator: tt} => {
        pub struct $type_name<S: SelectableTables, Lhs: SqlExpression<S, SqlType = SqlBool>, Rhs: SqlExpression<S, SqlType = SqlBool>>
        {
            lhs: Lhs,
            rhs: Rhs,
            phantom: PhantomData<S>,
        }

        impl<S: SelectableTables, Lhs: SqlExpression<S, SqlType = SqlBool>, Rhs: SqlExpression<S, SqlType = SqlBool>>
            $type_name<S, Lhs, Rhs>
        {
            pub fn new(lhs: Lhs, rhs: Rhs) -> Self {
                Self {
                    lhs,
                    rhs,
                    phantom: PhantomData,
                }
            }
        }

        impl<S: SelectableTables, Lhs: SqlExpression<S, SqlType = SqlBool>, Rhs: SqlExpression<S, SqlType = SqlBool>> SqlExpression<S>
            for $type_name<S, Lhs, Rhs>
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
                write!(f, " {} ", stringify!($operator))?;
                self.rhs.write_parenthesized_sql_string(f, parameter_binder)?;
                Ok(())
            }
        }
    }
}

define_boolean_binary_operator! {SqlBooleanAnd, AND}
define_boolean_binary_operator! {SqlBooleanOr, OR}

/// An sql `NOT` operator, which negates its boolean operand.
pub struct SqlNot<S: SelectableTables, E: SqlExpression<S, SqlType = SqlBool>> {
    expr: E,
    phantom: PhantomData<S>,
}

impl<S: SelectableTables, E: SqlExpression<S, SqlType = SqlBool>> SqlNot<S, E> {
    pub fn new(expr: E) -> Self {
        Self {
            expr,
            phantom: PhantomData,
        }
    }
}

impl<S: SelectableTables, E: SqlExpression<S, SqlType = SqlBool>> SqlExpression<S>
    for SqlNot<S, E>
{
    type SqlType = E::SqlType;
    type RustType = E::RustType;

    const IS_AGGREGATE: bool = E::IS_AGGREGATE;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "NOT")?;
        self.expr
            .write_parenthesized_sql_string(f, parameter_binder)
    }
}
