use super::ParameterBinder;
use crate::{statements::EmptyInsertStatement, Table};

/// A record type which can be inserted into the database.
#[async_trait::async_trait]
pub trait Insertable: Sized {
    type Table: Table;

    /// Writes the names of the values inserted by this insertable.
    /// For example, in the query:
    /// `INSERT INTO person(name,age) values('James', 29)`
    /// This represents the `name,age` part.
    fn write_value_names(&self, f: &mut String) -> std::fmt::Result;

    /// Writes the the values inserted by this insertable.
    /// For example, in the query:
    /// `INSERT INTO person(name,age) values('James', 29)`
    /// This represents the `'James',29` part.
    fn write_values<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Returns an insert statement for this record.
    fn insert(self) -> EmptyInsertStatement<Self> {
        EmptyInsertStatement::new(self)
    }
}
