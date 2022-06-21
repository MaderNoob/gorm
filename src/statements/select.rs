use std::marker::PhantomData;

use super::{select_from::SelectFrom, SelectFromFormatter, SqlStatement};
use crate::{condition::SqlCondition, table::{Table, TableMarker}};

/// An sql statement for finding records in a table
pub struct SelectStatement<S: SelectFrom>(PhantomData<S>);
impl<S: SelectFrom> SelectStatement<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn sql_from_string(self) -> String {
        SelectFromFormatter::<S>::new().to_string()
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
    fn write_sql_string(self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SELECT * FROM \"{}\"", T::Table::TABLE_NAME)
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
