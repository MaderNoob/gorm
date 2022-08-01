use std::{fmt::Write, marker::PhantomData};

use crate::sql::{ParameterBinder, SelectableTables, SqlBool, SqlExpression};

/// A condition in a where clause.
pub trait SqlCondition<S: SelectableTables> {
    /// Writes the condition as an sql string which can be used in a where
    /// clause.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;
}

macro_rules! define_operator_condition{
    {$type_name: ident, $operator: tt} => {
        pub struct $type_name<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S, SqlType = Lhs::SqlType>> {
            lhs: Lhs,
            rhs: Rhs,
            phantom: PhantomData<S>,
        }

        impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S, SqlType = Lhs::SqlType>>
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

        impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S, SqlType = Lhs::SqlType>> SqlCondition<S>
            for $type_name<S, Lhs, Rhs>
        {
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

        impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S, SqlType = Lhs::SqlType>> SqlExpression<S>
            for $type_name<S, Lhs, Rhs>
        {
            type SqlType = SqlBool;
            type RustType = bool;

            fn write_sql_string<'s, 'a>(
                &'s self,
                f: &mut String,
                parameter_binder: &mut ParameterBinder<'a>,
            ) -> std::fmt::Result
            where
                's: 'a,
            {
                <Self as SqlCondition<S>>::write_sql_string(self, f, parameter_binder)
            }
        }
    }
}

define_operator_condition! {SqlConditionEq, =}
define_operator_condition! {SqlConditionNotEq, !=}
define_operator_condition! {SqlConditionLowerThan, <}
define_operator_condition! {SqlConditionLowerEquals, <=}
define_operator_condition! {SqlConditionGreaterThan, >}
define_operator_condition! {SqlConditionGreaterEquals, >=}
