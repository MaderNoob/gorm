use std::fmt::Write;

use super::{
    AverageableSqlType, SqlAdd, SqlAddition, SqlAverage, SqlCount, SqlDivide, SqlDivision, SqlMax,
    SqlMultiplication, SqlMultiply, SqlSubtract, SqlSubtraction, SqlSum, SummableSqlType,
};
use crate::sql::{
    Column, IntoSqlType, OrderableSqlType, ParameterBinder, SelectableTables,
    SelectableTablesContains, SqlBool, SqlBooleanAnd, SqlBooleanOr, SqlConditionEq,
    SqlConditionGreaterEquals, SqlConditionGreaterThan, SqlConditionLowerEquals,
    SqlConditionLowerThan, SqlConditionNotEq, SqlText, SqlType, Table,
};

/// An sql expression
pub trait SqlExpression<S: SelectableTables>: Sized {
    type SqlType: SqlType;
    type RustType: IntoSqlType<SqlType = Self::SqlType>;

    const IS_AGGREGATE: bool;

    /// Writes the sql expression as an sql string which can be evaluated by the
    /// database.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes the sql expression as an sql string surrounded by parentheses.
    fn write_parenthesized_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "(")?;
        self.write_sql_string(f, parameter_binder)?;
        write!(f, ")")?;

        Ok(())
    }

    /// Returns a condition which will be true if the given expression is equal
    /// to this one.
    // only allow equality for expression with the same value type
    fn equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionEq<S, Self, O> {
        SqlConditionEq::new(self, other)
    }

    /// Returns a condition which will be true if the given expression is not
    /// equal to this one.
    // only allow equality for expression with the same value type
    fn not_equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionNotEq<S, Self, O> {
        SqlConditionNotEq::new(self, other)
    }

    /// Returns an expression which evaluates to the amount of items returned
    /// from the query.
    fn count(self) -> SqlCount<S, Self> {
        SqlCount::new(self)
    }
}

// a column is an sql expression
impl<S: SelectableTables, C: Column> SqlExpression<S> for C
where
    S: SelectableTablesContains<<C as Column>::Table>,
{
    type RustType = <C as Column>::RustType;
    type SqlType = <C as Column>::SqlType;

    const IS_AGGREGATE: bool = false;

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

pub trait OrderableSqlExpression<S: SelectableTables>: SqlExpression<S>
where
    Self::SqlType: OrderableSqlType,
{
    /// Returns a condition which will be true if this expression is lower than
    /// the given one.
    // only allow comparing with expression with the same value type
    fn lower_than<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionLowerThan<S, Self, O> {
        SqlConditionLowerThan::new(self, other)
    }

    /// Returns a condition which will be true if this expression is lower or
    /// equal to the given one.
    // only allow comparing with expression with the same value type
    fn lower_equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionLowerEquals<S, Self, O> {
        SqlConditionLowerEquals::new(self, other)
    }

    /// Returns a condition which will be true if this expression is greater
    /// than the given one.
    // only allow comparing with expression with the same value type
    fn greater_than<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionGreaterThan<S, Self, O> {
        SqlConditionGreaterThan::new(self, other)
    }

    /// Returns a condition which will be true if this expression is greater or
    /// equal to the given one.
    // only allow comparing with expression with the same value type
    fn greater_equals<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionGreaterEquals<S, Self, O> {
        SqlConditionGreaterEquals::new(self, other)
    }

    /// Returns an expression which evaluates to the max item of the items
    /// returned from the query.
    fn max(self) -> SqlMax<S, Self> {
        SqlMax::new(self)
    }
}
impl<S: SelectableTables, T: SqlExpression<S>> OrderableSqlExpression<S> for T where
    T::SqlType: OrderableSqlType
{
}

pub trait AverageableSqlExpression<S: SelectableTables>: SqlExpression<S>
where
    Self::SqlType: AverageableSqlType,
{
    /// Returns an expression which evaluates to the average of the items
    /// returned from the query.
    fn average(self) -> SqlAverage<S, Self> {
        SqlAverage::new(self)
    }
}
impl<S: SelectableTables, E: SqlExpression<S>> AverageableSqlExpression<S> for E where
    E::SqlType: AverageableSqlType
{
}

pub trait SummableSqlExpression<S: SelectableTables>: SqlExpression<S>
where
    Self::SqlType: SummableSqlType,
{
    /// Returns an expression which evaluates to the sum of the items
    /// returned from the query.
    fn sum(self) -> SqlSum<S, Self> {
        SqlSum::new(self)
    }
}
impl<S: SelectableTables, E: SqlExpression<S>> SummableSqlExpression<S> for E where
    E::SqlType: SummableSqlType
{
}

macro_rules! define_expression_operator_trait {
    {$trait_name: ident, $sql_type_marker: ident, $expr_type: ident, $fn_name: ident} => {
        pub trait $trait_name<S: SelectableTables, Rhs: SqlExpression<S>>:
            SqlExpression<S>
        where
            Self::SqlType: $sql_type_marker<Rhs::SqlType>,
        {
            fn $fn_name(self, other: Rhs) -> $expr_type<S, Self, Rhs> {
                $expr_type::new(self, other)
            }
        }

        impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S>> $trait_name<S, Rhs>
            for Lhs
        where
            Lhs::SqlType: $sql_type_marker<Rhs::SqlType>,
        {
        }
    };
}

define_expression_operator_trait! {AddableSqlExpression, SqlAdd, SqlAddition, add}
define_expression_operator_trait! {SubtractableSqlExpression, SqlSubtract, SqlSubtraction, subtract}
define_expression_operator_trait! {MultipliableSqlExpression, SqlMultiply, SqlMultiplication, multiply}
define_expression_operator_trait! {DivisibleSqlExpression, SqlDivide, SqlDivision, divide}

macro_rules! define_boolean_expression_operator_trait {
    {$trait_name: ident, $expr_type: ident, $fn_name: ident} => {
        pub trait $trait_name<S: SelectableTables, Rhs: SqlExpression<S, SqlType = SqlBool>>:
            SqlExpression<S, SqlType = SqlBool>
        {
            fn $fn_name(self, other: Rhs) -> $expr_type<S, Self, Rhs> {
                $expr_type::new(self, other)
            }
        }

        impl<S: SelectableTables, Lhs: SqlExpression<S, SqlType = SqlBool>, Rhs: SqlExpression<S, SqlType = SqlBool>> $trait_name<S, Rhs>
            for Lhs
        {
        }
    };
}

define_boolean_expression_operator_trait! {BooleanAndableSqlExpression, SqlBooleanAnd, and}
define_boolean_expression_operator_trait! {BooleanOrableSqlExpression, SqlBooleanOr, or}

macro_rules! impl_primitive_expression{
    { $($t: ty),+ }=> {
        $(
            impl<S: SelectableTables> SqlExpression<S> for $t
            {
                type SqlType = <$t as IntoSqlType>::SqlType;
                type RustType = $t;

                const IS_AGGREGATE:bool = false;

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
    type RustType = &'b str;
    type SqlType = SqlText;

    const IS_AGGREGATE: bool = false;

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
