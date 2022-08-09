use super::{Column, FieldNameCharsConsListItem, FieldsConsListCons, SelectableTablesContains};
use crate::{
    sql::{FieldsConsListItem, ParameterBinder, SelectableTables},
    util::TypedConsListNil,
};

/// The selected values in an sql query.
pub trait SelectedValues<S: SelectableTables> {
    type Fields: FieldsConsListItem;

    const IS_AGGREGATE: bool;

    /// Writes the selected values as an sql string which can be selected by the
    /// database.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;
}

/// A marker trait which indicates that the list of selected values contains a
/// field with the given name.
pub trait SelectedValuesContainsFieldWithName<N: FieldNameCharsConsListItem> {}

// A table column is considered a `SelectedValues`.
impl<S: SelectableTables + SelectableTablesContains<C::Table>, C: Column> SelectedValues<S> for C {
    type Fields = FieldsConsListCons<C::ColumnName, C::RustType, TypedConsListNil>;

    const IS_AGGREGATE: bool = false;

    /// Writes the selected values as an sql string which can be selected by the
    /// database.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        use std::fmt::Write;

        write!(f, "{}", C::COLUMN_NAME)
    }
}
