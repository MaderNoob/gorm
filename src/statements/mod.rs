mod create_table;
mod drop_table;
mod select;
mod select_from;

use crate::error::*;
use std::fmt::Display;

use async_trait::async_trait;
pub use create_table::*;
pub use drop_table::*;
pub use select::*;
pub use select_from::*;

use crate::{execution::SqlStatementExecutor, fields_list::FieldsConsListItem};

/// An sql statement which can be executed by a database.
pub trait SqlStatement: Sized + 'static {
    /// The fields of the output of this statement.
    type OutputFields: FieldsConsListItem;

    /// Writes the sql statement as an sql string which can be executed by a database.
    fn write_sql_string(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;

    /// Returns a formatter for this sql statement which implmenets the `Display` trait for it, and
    /// when displayed, writes the sql string corresponding with this statement.
    fn formatter(&self) -> SqlStatementFormatter<Self> {
        SqlStatementFormatter { statement: self }
    }
}

#[async_trait(?Send)]
pub trait ExecuteSqlStatment: SqlStatement {
    /// Executes the query on the given executor
    async fn execute<E: SqlStatementExecutor>(self, on: E) -> Result<()> {
        on.execute(self).await
    }
}

#[async_trait(?Send)]
impl<S: SqlStatement> ExecuteSqlStatment for S {}

/// An sql statement formatter which implemented `Display` for some sql statement type.
pub struct SqlStatementFormatter<'a, S: SqlStatement> {
    statement: &'a S,
}
impl<'a, S: SqlStatement> Display for SqlStatementFormatter<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.statement.write_sql_string(f)
    }
}
