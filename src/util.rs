use crate::{
    fields_list::{FieldNameCharsConsListItem, FieldsConsListItem},
    selectable_tables::SelectableTables,
    selected_values::SelectedValuesConsListItem,
};

/// A marker trait for marking 2 types not the same.
pub auto trait TypesNotEqual {}
impl<T> !TypesNotEqual for (T, T) {}

/// A nil item of a types cons list.
pub struct TypedConsListNil;
impl FieldsConsListItem for TypedConsListNil {}
impl FieldNameCharsConsListItem for TypedConsListNil {}
impl<S: SelectableTables> SelectedValuesConsListItem<S> for TypedConsListNil {
    type Next = TypedConsListNil;

    // the value of this doesn't matter, since this item is a nil and will panic when asked for its
    // `cur_expr`.
    type SqlExpression = i32;

    fn cur_expr(&self) -> &crate::selected_values::NamedSelectedExpression<S, Self::SqlExpression> {
        panic!("can't get the expression of a nil value");
    }

    fn next_item(&self) -> &Self::Next {
        panic!("can't get the next item of a nil value");
    }
}
