use std::{fmt::Write, marker::PhantomData};

use crate::{
    sql::{ParameterBinder, Table},
    statements::SqlStatement,
    util::TypedConsListNil,
};

/// An sql drop table statement
///
/// This statement can be created by calling the [`TableMarker::drop`]
/// function.
pub struct DropTableStatement<T: Table>(PhantomData<T>);
impl<T: Table> DropTableStatement<T> {
    /// Creates a new sql drop table statement.
    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// Do not throw an error if the table does not exist.
    pub fn if_exists(self) -> DropTableIfExistsStatement<T> {
        DropTableIfExistsStatement(PhantomData)
    }
}
impl<T: Table> SqlStatement for DropTableStatement<T> {
    type OutputFields = TypedConsListNil;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "DROP TABLE {}", T::TABLE_NAME)
    }
}

/// An sql drop table if exists statement.
///
/// This statement doesn't return an error in case the table does not exist.
///
/// This statement can be created by calling the [`DropTableStatement::if_exists`]
/// function.
pub struct DropTableIfExistsStatement<T: Table>(PhantomData<T>);
impl<T: Table> SqlStatement for DropTableIfExistsStatement<T> {
    type OutputFields = TypedConsListNil;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "DROP TABLE IF EXISTS {}", T::TABLE_NAME)
    }
}
