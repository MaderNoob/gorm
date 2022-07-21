use std::fmt::Write;
use std::marker::PhantomData;

use super::{select_from::SelectFrom, SqlStatement};
use crate::{
    bound_parameters::ParameterBinder, condition::SqlCondition, table::TableMarker, Table,
};

/// An sql statement for finding records in a table
pub struct SelectStatement<S: SelectFrom>(PhantomData<S>);
impl<S: SelectFrom> SelectStatement<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn filter<C: SqlCondition<<S as SelectFrom>::SelectableTables>>(
        self,
        condition: C,
    ) -> FilteredSelectStatement<S, C> {
        FilteredSelectStatement {
            condition,
            phantom: PhantomData,
        }
    }
}
impl<T: TableMarker> SqlStatement for SelectStatement<T> {
    type OutputFields = <T::Table as Table>::Fields;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a{
        write!(f, "SELECT * FROM \"{}\"", <T::Table as Table>::TABLE_NAME)
    }
}

/// An sql statement for finding records in a table matching some condition
pub struct FilteredSelectStatement<
    S: SelectFrom,
    C: SqlCondition<<S as SelectFrom>::SelectableTables>,
> {
    condition: C,
    phantom: PhantomData<S>,
}

impl<T: TableMarker, C: SqlCondition<<T as SelectFrom>::SelectableTables> + 'static> SqlStatement
    for FilteredSelectStatement<T, C>
{
    type OutputFields = <T::Table as Table>::Fields;

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
            "SELECT * FROM \"{}\" where ",
            <T::Table as Table>::TABLE_NAME
        )?;
        self.condition
            .write_sql_string(f, parameter_binder)
    }
}
