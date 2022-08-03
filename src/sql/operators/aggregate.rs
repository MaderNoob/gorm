use std::{fmt::Write, marker::PhantomData};

use rust_decimal::Decimal;

use crate::sql::{
    AverageableSqlType, OrderableSqlType, ParameterBinder, SelectableTables, SqlExpression, SqlI64,
    SqlNumeric, SqlType, SummableSqlType,
};

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

