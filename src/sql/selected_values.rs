use crate::sql::{FieldsConsListItem, ParameterBinder, SelectableTables};

use super::FieldNameCharsConsListItem;

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

/// A marker trait which indicates that the list of selected values contains a field with the given
/// name.
pub trait SelectedValuesContainsFieldWithName<N: FieldNameCharsConsListItem>{}
