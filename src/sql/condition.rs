use std::{fmt::Write, marker::PhantomData};

use crate::sql::{OrderableSqlType, ParameterBinder, SelectableTables, SqlBool, SqlExpression};

/// Defines an operator condition struct.
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

        impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S, SqlType = Lhs::SqlType>> SqlExpression<S>
            for $type_name<S, Lhs, Rhs>
        {
            type SqlType = SqlBool;
            type RustType = bool;

            const IS_AGGREGATE:bool = Lhs::IS_AGGREGATE || Rhs::IS_AGGREGATE;

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

define_operator_condition! {SqlConditionEq, =}
define_operator_condition! {SqlConditionNotEq, !=}

macro_rules! define_operator_condition_orderable{
    {$type_name: ident, $operator: tt} => {
        pub struct $type_name<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S, SqlType = Lhs::SqlType>>
        where
            Lhs::SqlType: OrderableSqlType
        {
            lhs: Lhs,
            rhs: Rhs,
            phantom: PhantomData<S>,
        }

        impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S, SqlType = Lhs::SqlType>>
            $type_name<S, Lhs, Rhs>
        where
            Lhs::SqlType: OrderableSqlType
        {
            pub fn new(lhs: Lhs, rhs: Rhs) -> Self {
                Self {
                    lhs,
                    rhs,
                    phantom: PhantomData,
                }
            }
        }

        impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S, SqlType = Lhs::SqlType>> SqlExpression<S>
            for $type_name<S, Lhs, Rhs>
        where
            Lhs::SqlType: OrderableSqlType
        {
            type SqlType = SqlBool;
            type RustType = bool;

            const IS_AGGREGATE:bool = Lhs::IS_AGGREGATE || Rhs::IS_AGGREGATE;

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

define_operator_condition_orderable! {SqlConditionLowerThan, <}
define_operator_condition_orderable! {SqlConditionLowerEquals, <=}
define_operator_condition_orderable! {SqlConditionGreaterThan, >}
define_operator_condition_orderable! {SqlConditionGreaterEquals, >=}
