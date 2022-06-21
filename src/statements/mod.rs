mod create_table;
mod drop_table;
mod select;
mod select_from;

use std::fmt::Display;

pub use create_table::*;
pub use drop_table::*;
pub use select::*;
pub use select_from::*;

/// An sql statement which can be executed on the database.
pub trait SqlStatement : Sized {
    /// Writes the sql statement as an sql string which can be executed on the database.
    fn write_sql_string(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;

    fn formatter(&self) -> SqlStatementFormatter<Self> {
        SqlStatementFormatter { statement: self }
    }
}

/// An sql statement formatter which implemented `Display` for some sql statement type.
pub struct SqlStatementFormatter<'a, S: SqlStatement> {
    statement: &'a S,
}
impl<'a, S: SqlStatement> Display for SqlStatementFormatter<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.statement.write_sql_string(f)
    }
}
