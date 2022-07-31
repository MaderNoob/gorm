use std::marker::PhantomData;

use crate::{
    bound_parameters::ParameterBinder, expr::SqlExpression, fields_list::FieldsConsListItem,
    selectable_tables::SelectableTables, SelectFrom,
};

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

pub trait SelectedValuesConsListItem<S: SelectableTables> {
    /// The next item.
    type Next: SelectedValuesConsListItem<S>;

    /// The sql expression of this current item.
    /// Only relevant if the item is not a nil.
    type SqlExpression: SqlExpression<S>;

    /// Returns a reference to the current expression.
    /// If the item is a nil, this will panic.
    fn cur_expr(&self) -> &NamedSelectedExpression<S, Self::SqlExpression>;

    /// Returns a reference to the next item in the cons list.
    /// If the item is a nil, this will panic.
    fn next_item(&self) -> &Self::Next;
}
pub struct SelectedValuesConsListCons<
    S: SelectableTables,
    Cur: SqlExpression<S>,
    Next: SelectedValuesConsListItem<S>,
> {
    pub cur: NamedSelectedExpression<S, Cur>,
    pub next: Next,
    pub phantom: PhantomData<S>,
}
impl<S: SelectableTables, Cur: SqlExpression<S>, Next: SelectedValuesConsListItem<S>>
    SelectedValuesConsListItem<S> for SelectedValuesConsListCons<S, Cur, Next>
{
    type Next = Next;

    type SqlExpression = Cur;

    fn cur_expr(&self) -> &NamedSelectedExpression<S, Self::SqlExpression> {
        &self.cur
    }

    fn next_item(&self) -> &Self::Next {
        &self.next
    }
}

pub struct NamedSelectedExpression<S: SelectableTables, E: SqlExpression<S>> {
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
macro_rules! format_selected_sql_expressoins {
    {$cur_item:expr, $f: expr, $parameter_binder: expr, $head:expr} => {
        let named_expr = $cur_item.cur_expr();
        named_expr.expr.write_sql_string($f, $parameter_binder)?;
        write!($f,", ")?;
    };
    {$cur_item: expr, $f: expr, $parameter_binder: expr, $head:expr, $($tail: expr),*} => {
        let named_expr = $cur_item.cur_expr();
        named_expr.expr.write_sql_string($f, $parameter_binder)?;
        write!($f,", ")?;
        let cur_item = $cur_item.next_item();
        ::gorm::format_selected_sql_expressoins!{cur_item, $f, $parameter_binder, $($tail),*}
    };
}

// pub struct FieldsConsListCons<
//     FieldName: FieldNameCharsConsListItem,
//     FieldType,
//     Next: FieldsConsListItem,
// >(
//     PhantomData<FieldName>,
//     PhantomData<FieldType>,
//     PhantomData<Next>,
// );

//  #[macro_export]
//  macro_rules! create_selected_values_fields_cons_list {
//      {$name:ident : $type: path} => {
//          ::gorm::fields_list::FieldsConsListCons<::gorm::create_field_name_cons_list!{$name}, $type, ::gorm::TypedConsListNil>
//      };
//      {$name:ident : $type: path, $($rest_name: ident : $rest_type: path),+} => {
//          ::gorm::fields_list::FieldsConsListCons<::gorm::create_field_name_cons_list!{$name}, $type, create_selected_values_fields_cons_list!{$($rest_name : $rest_type),+}>
//      }
//  }

#[macro_export]
macro_rules! create_custom_selected_values_generic_expr_types {
     {$($value: expr),+} => {
        struct X<
            ::gorm::create_custom_selected_values_generic_expr_types!{E; $($value),+}
        >
     };
     {$cur_name:ident; $value:expr $(,$rest: expr)+} => {
        ::gorm::paste::paste!{
            $cur_name: SqlExpression<S>, ::gorm::create_custom_selected_values_generic_expr_types!{[<$cur_name E>], $($rest),+}
        }
     };
 }

// #[macro_export]
// macro_rules! select_values {
//     ($(($value: expr) as $name:ident),+) => {
//         {
//             ::gorm::create_custom_selected_values_generic_expr_types!{ $($value),+ }
//         struct CustomSelectedValues<
//             S: ::gorm::selectable_tables::SelectableTables,
//             ConsList: ::gorm::selected_values::SelectedValuesConsListItem<S>
//         > {
//             cons_list: ConsList,
//             phantom: PhantomData<S>,
//         }
// 
//         impl<
//             S: ::gorm::selectable_tables::SelectableTables,
//             ConsList: ::gorm::selected_values::SelectedValuesConsListItem<S>
//         > ::gorm::selected_values::SelectedValues for CustomSelectedValues<S, ConsList> {
//                 type Fields = ::gorm::create_selected_values_fields_cons_list!{$($name : $value),+};
// 
//                 /// Writes the selected values as an sql string which can be selected by the database.
//                 fn write_sql_string<'s, 'a>(
//                     &'s self,
//                     f: &mut ::std::string::String,
//                     parameter_binder: &mut ::gorm::bound_parameters::ParameterBinder<'a>,
//                 ) -> ::std::fmt::Result
//                 where
//                     's: 'a{
//                         use ::gorm::expr::SqlExpression;
//                         use ::std::fmt::Write;
//                         use ::gorm::selected_values::SelectedValuesConsListItem;
// 
//                         let cur_item = &self.cons_list;
//                         ::gorm::format_selected_sql_expressoins!{cur_item, f, parameter_binder, $($value),+}
// 
//                         Ok(())
//                     }
//             }
// 
//             let cons_list = ::gorm::create_selected_values_cons_list!($(($value) as $name),+);
//             CustomSelectedValues{
//                 cons_list,
//                 phantom: ::std::marker::PhantomData,
//             }
//         }
//     };
// }
