use crate::sql::{FieldsConsListItem, ParameterBinder, SelectableTables};

/// The selected values in an sql query.
pub trait SelectedValues<S: SelectableTables> {
    type Fields: FieldsConsListItem;

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
