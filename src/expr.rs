use sqlx::{database::HasArguments, query::QueryAs, Database, Encode, Type};

use crate::{
    bound_parameters::BoundParametersFormatter,
    condition::SqlConditionEq,
    selectable_tables::{SelectableTables, SelectableTablesContains},
    table::{Column, Table},
    types::{IntoSqlType, SqlType},
};

/// An sql expression
pub trait SqlExpression<S: SelectableTables>: Sized {
    type SqlType: SqlType;
    type RustType;

    /// Writes the sql statement as an sql string which can be executed on the database.
    fn write_sql_string(
        &self,
        f: &mut std::fmt::Formatter,
        bound_parameters_formatter: &mut impl BoundParametersFormatter,
    ) -> std::fmt::Result;

    /// Binds the parameters of this sql expression
    fn bind_parameters<'s, 'q, DB: Database, O>(
        &'s self,
        q: QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>,
    ) -> QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>
    where
        Self::RustType: Type<DB> + Encode<'q, DB>,
        's: 'q,
    {
        q
    }

    /// Returns a condition which will be true if the given expression it equal to this one.
    // only allow equality for expression with the same value type
    fn eq<O: SqlExpression<S, SqlType = <Self as SqlExpression<S>>::SqlType>>(
        &self,
        _other: O,
    ) -> SqlConditionEq<S, Self, O> {
        SqlConditionEq::new()
    }
}

// a column is an sql expression
impl<S: SelectableTables, C: Column> SqlExpression<S> for C
where
    S: SelectableTablesContains<<C as Column>::Table>,
{
    type SqlType = <C as Column>::SqlType;
    type RustType = <C as Column>::RustType;

    fn write_sql_string(
        &self,
        f: &mut std::fmt::Formatter,
        _bound_parameters_formatter: &mut impl BoundParametersFormatter,
    ) -> std::fmt::Result {
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

                fn write_sql_string(
                    &self,
                    f: &mut std::fmt::Formatter,
                    bound_parameters_formatter: &mut impl BoundParametersFormatter
                ) -> std::fmt::Result {
                    write!(
                        f,
                        "{}",
                        bound_parameters_formatter.format_next_bound_parameter()
                    )
                }

                fn bind_parameters<'s, 'q, DB: Database, O>(
                    &'s self,
                    q: QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>,
                ) -> QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>
                    where Self::RustType: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
                          's: 'q
                {
                    q.bind(self)
                }
            }
        )+
    }
}

impl_primitive_expression! {bool, i16, i32, i64, f32, f64}
