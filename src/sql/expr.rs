use std::fmt::Write;

use crate::{
    bound_parameters::ParameterBinder,
    condition::SqlConditionEq,
    selectable_tables::{SelectableTables, SelectableTablesContains},
    table::{Column, Table},
    types::{IntoSqlType, SqlText, SqlType},
};

/// An sql expression
pub trait SqlExpression<S: SelectableTables> : Sized {
    type SqlType: SqlType;
    type RustType;

    /// Writes the sql expression as an sql string which can be evaluated by the database.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Returns a condition which will be true if the given expression it equal to this one.
    // only allow equality for expression with the same value type
    fn eq<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        self,
        other: O,
    ) -> SqlConditionEq<S, Self, O> {
        SqlConditionEq::new(self, other)
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
