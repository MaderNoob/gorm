use std::marker::PhantomData;

use crate::{table::{Column, Table}, util::TypesNotEqual};

/// A list of tables from which columns can be used in an sql expression.
pub trait SelectableTables {
    /// Writes the `from` part of the sql query as an sql string.
    fn write_sql_from_string(self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

/// A marker traits which indicates that a list of tables contains some table.
pub trait SelectableTablesContains<T: Table>: SelectableTables {}

/// A cons item in the selectable tables cons list.
pub struct SelectableTablesCons<T: Table, N: SelectableTables>(PhantomData<T>, PhantomData<N>);

// A table is a `SelectableTables`, and as such, it contains itself
impl<T: Table> SelectableTables for T {
    fn write_sql_from_string(self, f: &mut std::fmt::Formatter) -> std::fmt::Result{

    }
}
impl<T: Table> SelectableTablesContains<T> for T {}

// a cons list item is a `SelectableTables`
impl<T: Table, N: SelectableTables> SelectableTables for SelectableTablesCons<T, N> {}

// a cons list item contains the Table it holds, and everything else that its next holds which is
// not the same as the table it holds.
impl<T: Table, N: SelectableTables> SelectableTablesContains<T> for SelectableTablesCons<T, N> {}
impl<T: Table, N: SelectableTables, InnerT: Table> SelectableTablesContains<InnerT>
    for SelectableTablesCons<T, N>
where
    N: SelectableTablesContains<InnerT>,
    (T, InnerT): TypesNotEqual,
{
}

