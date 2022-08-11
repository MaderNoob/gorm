use super::ParameterBinder;
use crate::Table;

/// A set of updates to be performed on some row of a table.
pub trait UpdateSet {
    /// The table which this update set operates on.
    type UpdateTable: Table;

    /// Writes the update set as a comma seperated list of assignments to
    /// columns of the table.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;
}
