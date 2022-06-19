use crate::{selectable_tables::SelectableTables, table::Table};

/// Something which you can select from.
/// This can be a table or multiple joined tables.
pub trait SelectFrom{
    type SelectableTables: SelectableTables;
}

impl<T:Table> SelectFrom for T{
    type SelectableTables = T;
}

