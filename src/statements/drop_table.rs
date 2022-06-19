use std::marker::PhantomData;

use crate::{table::Table, statements::SqlStatement};

/// An sql drop table statement
pub struct DropTableStatement<T: Table>(PhantomData<T>);
impl<T: Table> DropTableStatement<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// Do not throw an error if the table does not exist.
    pub fn if_exists(self) -> DropTableIfExistsStatement<T> {
        DropTableIfExistsStatement(PhantomData)
    }
}
impl<T: Table> SqlStatement for DropTableStatement<T> {
    fn write_sql_string(self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DROP TABLE {}", T::TABLE_NAME)
    }
}

/// An sql drop table if exists statement
pub struct DropTableIfExistsStatement<T: Table>(PhantomData<T>);
impl<T: Table> SqlStatement for DropTableIfExistsStatement<T> {
    fn write_sql_string(self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DROP TABLE IF EXISTS {}", T::TABLE_NAME)
    }
}

