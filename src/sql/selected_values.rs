use std::marker::PhantomData;

use crate::{bound_parameters::ParameterBinder, fields_list::FieldsConsListItem, expr::SqlExpression, SelectFrom, selectable_tables::SelectableTables};

/// The selected values in an sql query.
pub trait SelectedValues {
    type Fields: FieldsConsListItem;

    /// Writes the selected values as an sql string which can be selected by the database.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;
}

pub trait SelectedValuesConsListItem<S: SelectableTables> {}
pub struct SelectedValuesConsListCons<S: SelectableTables, Cur: SqlExpression<S>, Next: SelectedValuesConsListItem<S>> {
    pub cur: NamedSelectedExpression<S, Cur>,
    pub next: Next,
    pub phantom: PhantomData<S>,
}
impl<S: SelectableTables, Cur: SqlExpression<S>, Next: SelectedValuesConsListItem<S>> SelectedValuesConsListItem<S> for SelectedValuesConsListCons<S, Cur, Next> {}

pub struct NamedSelectedExpression<S: SelectableTables, E: SqlExpression<S>>{
    pub expr: E,
    pub name: &'static str,
    pub phantom: PhantomData<S>,
}


#[macro_export]
macro_rules! create_selected_values_cons_list {
    () => {
        ::gorm::TypedConsListNil
    };
    (($head: expr) as $head_name:ident, $(($tail: expr) as $tail_name:ident),*) => {
        ::gorm::selected_values::SelectedValuesConsListCons{
            cur: ::gorm::selected_values::NamedSelectedExpression{
                expr: $head,
                name: stringify!($head_name),
                phantom: PhantomData,
            },
            next: ::gorm::create_selected_values_cons_list!($(($tail) as $tail_name,)*),
            phantom: ::std::marker::PhantomData,
        }
    }
}

#[macro_export]
macro_rules! select_values {
    ($(($value: expr) as $name:ident),+) => {
        {
            let cons_list = ::gorm::create_selected_values_cons_list!($(($value) as $name),+);
            cons_list
        }
    };
}
