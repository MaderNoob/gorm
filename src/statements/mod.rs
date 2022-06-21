mod create_table;
mod drop_table;
mod select;
mod select_from;

pub use create_table::*;
pub use drop_table::*;
pub use select::*;
pub use select_from::*;

/// An sql statement which can be executed on the database.
pub trait SqlStatement {
    /// Writes the sql statement as an sql string which can be executed on the database.
    fn write_sql_string(self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}
