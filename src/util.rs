use crate::{fields_list::{FieldsConsListItem, FieldNameCharsConsListItem}, selected_values::SelectedValuesConsListItem, selectable_tables::SelectableTables};

/// A marker trait for marking 2 types not the same.
pub auto trait TypesNotEqual {}
impl<T> !TypesNotEqual for (T, T) {}

/// A nil item of a types cons list.
pub struct TypedConsListNil;
impl FieldsConsListItem for TypedConsListNil {}
impl FieldNameCharsConsListItem for TypedConsListNil {}
impl<S: SelectableTables> SelectedValuesConsListItem<S> for TypedConsListNil {}
