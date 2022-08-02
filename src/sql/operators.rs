use std::{fmt::Write, marker::PhantomData};

use rust_decimal::Decimal;

use super::{
    AggregateSqlExpression, AverageableSqlType, OrderableSqlType, SqlAdd, SqlDivide, SqlI64,
    SqlMultiply, SqlNumeric, SqlSubtract, SqlType, SummableSqlType,
};
use crate::sql::{NonAggregateSqlExpression, ParameterBinder, SelectableTables, SqlExpression};

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

impl<S: SelectableTables, E: SqlExpression<S>> SqlExpression<S> for SqlCount<S, E> {
    type RustType = i64;
    type SqlType = SqlI64;

    const IS_AGGREGATE: bool = true;

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

impl<S: SelectableTables, E: SqlExpression<S>> AggregateSqlExpression for SqlCount<S, E> {}

pub struct SqlAverage<S: SelectableTables, E: SqlExpression<S>>
where
    E::SqlType: AverageableSqlType,
{
    expr: E,
    phantom: PhantomData<S>,
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlAverage<S, E>
where
    E::SqlType: AverageableSqlType,
{
    pub fn new(expr: E) -> Self {
        Self {
            expr,
            phantom: PhantomData,
        }
    }
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlExpression<S> for SqlAverage<S, E>
where
    E::SqlType: AverageableSqlType,
{
    type RustType = Decimal;
    type SqlType = SqlNumeric;

    const IS_AGGREGATE: bool = true;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "AVG(")?;
        self.expr.write_sql_string(f, parameter_binder)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl<S: SelectableTables, E: SqlExpression<S>> AggregateSqlExpression for SqlAverage<S, E> where
    E::SqlType: AverageableSqlType
{
}

pub struct SqlSum<S: SelectableTables, E: SqlExpression<S>>
where
    E::SqlType: SummableSqlType,
{
    expr: E,
    phantom: PhantomData<S>,
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlSum<S, E>
where
    E::SqlType: SummableSqlType,
{
    pub fn new(expr: E) -> Self {
        Self {
            expr,
            phantom: PhantomData,
        }
    }
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlExpression<S> for SqlSum<S, E>
where
    E::SqlType: SummableSqlType,
{
    type RustType = <<E::SqlType as SummableSqlType>::OutputSqlType as SqlType>::RustType;
    type SqlType = <E::SqlType as SummableSqlType>::OutputSqlType;

    const IS_AGGREGATE: bool = true;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "SUM(")?;
        self.expr.write_sql_string(f, parameter_binder)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl<S: SelectableTables, E: SqlExpression<S>> AggregateSqlExpression for SqlSum<S, E> where
    E::SqlType: SummableSqlType
{
}

pub struct SqlMax<S: SelectableTables, E: SqlExpression<S>>
where
    E::SqlType: OrderableSqlType,
{
    expr: E,
    phantom: PhantomData<S>,
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlMax<S, E>
where
    E::SqlType: OrderableSqlType,
{
    pub fn new(expr: E) -> Self {
        Self {
            expr,
            phantom: PhantomData,
        }
    }
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlExpression<S> for SqlMax<S, E>
where
    E::SqlType: OrderableSqlType,
{
    type RustType = E::RustType;
    type SqlType = E::SqlType;

    const IS_AGGREGATE: bool = true;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "MAX(")?;
        self.expr.write_sql_string(f, parameter_binder)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl<S: SelectableTables, E: SqlExpression<S>> AggregateSqlExpression for SqlMax<S, E> where
    E::SqlType: OrderableSqlType
{
}

pub struct SqlMin<S: SelectableTables, E: SqlExpression<S>>
where
    E::SqlType: OrderableSqlType,
{
    expr: E,
    phantom: PhantomData<S>,
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlMin<S, E>
where
    E::SqlType: OrderableSqlType,
{
    pub fn new(expr: E) -> Self {
        Self {
            expr,
            phantom: PhantomData,
        }
    }
}

impl<S: SelectableTables, E: SqlExpression<S>> SqlExpression<S> for SqlMin<S, E>
where
    E::SqlType: OrderableSqlType,
{
    type RustType = E::RustType;
    type SqlType = E::SqlType;

    const IS_AGGREGATE: bool = true;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "MIN(")?;
        self.expr.write_sql_string(f, parameter_binder)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl<S: SelectableTables, E: SqlExpression<S>> AggregateSqlExpression for SqlMin<S, E> where
    E::SqlType: OrderableSqlType
{
}
