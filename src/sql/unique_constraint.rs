use crate::Table;

/// A trait used to represent some unique constraint on some table.
///
/// This trait is used to refer to unique constraints in `ON CONFLICT` clauses.
pub trait UniqueConstraint {
    /// The table that this unique constraint belongs to.
    type Table: Table;

    /// A comma seperated list of field names that this unique constraint
    /// enforces uniqueness on.
    const FIELDS_COMMA_SEPERATED: &'static str;
}
