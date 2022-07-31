use std::fmt::Write;

use crate::{
    bound_parameters::ParameterBinder,
    condition::{
        SqlConditionEq, SqlConditionGreaterEquals, SqlConditionGreaterThan,
        SqlConditionLowerEquals, SqlConditionLowerThan, SqlConditionNotEq,
    },
    selectable_tables::{SelectableTables, SelectableTablesContains},
    table::{Column, Table},
    types::{IntoSqlType, OrderableSqlType, SqlText, SqlType},
};

/// An sql expression
pub trait SqlExpression<S: SelectableTables>: Sized {
    type SqlType: SqlType;
    type RustType: IntoSqlType;

    /// Writes the sql expression as an sql string which can be evaluated by the database.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Returns a condition which will be true if the given expression is equal to this one.
    // only allow equality for expression with the same value type
    fn equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionEq<S, Self, O> {
        SqlConditionEq::new(self, other)
    }

    /// Returns a condition which will be true if the given expression is not equal to this one.
    // only allow equality for expression with the same value type
    fn not_equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionNotEq<S, Self, O> {
        SqlConditionNotEq::new(self, other)
    }
}

// a column is an sql expression
impl<S: SelectableTables, C: Column> SqlExpression<S> for C
where
    S: SelectableTablesContains<<C as Column>::Table>,
{
    type SqlType = <C as Column>::SqlType;
    type RustType = <C as Column>::RustType;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(
            f,
            "\"{}\".\"{}\"",
            <<C as Column>::Table as Table>::TABLE_NAME,
            C::COLUMN_NAME,
        )
    }
}

pub trait OrderableSqlExpression<S: SelectableTables>: SqlExpression<S> {
    /// Returns a condition which will be true if this expression is lower than the given one.
    // only allow comparing with expression with the same value type
    fn lower_than<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionLowerThan<S, Self, O>;

    /// Returns a condition which will be true if this expression is lower or equal to the given one.
    // only allow comparing with expression with the same value type
    fn lower_equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionLowerEquals<S, Self, O>;

    /// Returns a condition which will be true if this expression is greater than the given one.
    // only allow comparing with expression with the same value type
    fn greater_than<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionGreaterThan<S, Self, O>;

    /// Returns a condition which will be true if this expression is greater or equal to the given one.
    // only allow comparing with expression with the same value type
    fn greater_equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionGreaterEquals<S, Self, O>;
}
impl<S: SelectableTables, T: SqlExpression<S>> OrderableSqlExpression<S> for T
where
    T::SqlType: OrderableSqlType,
{
    fn lower_than<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionLowerThan<S, Self, O> {
        SqlConditionLowerThan::new(self, other)
    }

    fn lower_equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionLowerEquals<S, Self, O> {
        SqlConditionLowerEquals::new(self, other)
    }

    fn greater_than<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionGreaterThan<S, Self, O> {
        SqlConditionGreaterThan::new(self, other)
    }

    fn greater_equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionGreaterEquals<S, Self, O> {
        SqlConditionGreaterEquals::new(self, other)
    }
}

// define_operator_condition!{SqlConditionEq, =}
// define_operator_condition!{SqlConditionNotEq, !=}
// define_operator_condition!{SqlConditionLowerThan, <}
// define_operator_condition!{SqlConditionLowerEquals, <=}
// define_operator_condition!{SqlConditionGreaterThan, >}
// define_operator_condition!{SqlConditionGreaterEquals, >=}

macro_rules! impl_primitive_expression{
    { $($t: ty),+ }=> {
        $(
            impl<S: SelectableTables> SqlExpression<S> for $t
            {
                type SqlType = <$t as IntoSqlType>::SqlType;
                type RustType = $t;

                fn write_sql_string<'s, 'a>(
                    &'s self,
                    f: &mut String,
                    parameter_binder: &mut ParameterBinder<'a>,
                ) -> std::fmt::Result
                where
                    's: 'a,
                {
                    write!(
                        f,
                        "{}",
                        parameter_binder.bind_parameter(self)
                    )
                }
            }
        )+
    }
}

impl_primitive_expression! {bool, i16, i32, i64, f32, f64}

impl<'b, S: SelectableTables> SqlExpression<S> for &'b str {
    type SqlType = SqlText;
    type RustType = &'b str;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "{}", parameter_binder.bind_parameter(self))
    }
}
