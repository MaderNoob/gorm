use std::marker::PhantomData;

use crate::{condition::SqlCondition, selectable_tables::SelectableTables, table::Table};

use super::SqlStatement;

/// An sql statement for finding records in a table
pub struct FindStatement<T: Table>(PhantomData<T>);
impl<T: Table> FindStatement<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn filter<C: SqlCondition<T>>(self, condition: C) -> FilteredFindStatement<T, C> {
        FilteredFindStatement {
            condition,
            phantom: PhantomData,
        }
    }
}
impl<T: Table> SqlStatement for FindStatement<T> {
    fn write_sql_string(self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SELECT * FROM \"{}\"", T::TABLE_NAME)
    }
}

/// An sql statement for finding records in a table matching some condition
pub struct FilteredFindStatement<S: SelectableTables, C: SqlCondition<S>> {
    condition: C,
    phantom: PhantomData<S>,
}
