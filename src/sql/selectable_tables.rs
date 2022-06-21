use std::marker::PhantomData;

use crate::{table::Table, util::TypesNotEqual};

/// A list of tables from which columns can be used in an sql expression.
pub trait SelectableTables {}

/// A marker traits which indicates that a list of tables contains some table.
pub trait SelectableTablesContains<T: Table>: SelectableTables {}

/// A cons item in the selectable tables cons list.
pub struct SelectableTablesCons<T: Table, N: SelectableTables>(PhantomData<T>, PhantomData<N>);

// A table is a `SelectableTables`, and as such, it contains itself
impl<T: Table> SelectableTables for T {}
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

/// A trait used to combine 2 selectable tables.
pub trait CombineSelectableTables<CombineWith: SelectableTables> {
    type Combined: SelectableTables;
}

impl<T: Table + SelectableTables, CombineWith: SelectableTables>
    CombineSelectableTables<CombineWith> for T
{
    type Combined = SelectableTablesCons<T, CombineWith>;
}

pub type CombinedSelectableTables<A, B> = <A as CombineSelectableTables<B>>::Combined;

impl<
        T: Table,
        N: SelectableTables + CombineSelectableTables<CombineWith>,
        CombineWith: SelectableTables,
    > CombineSelectableTables<CombineWith> for SelectableTablesCons<T, N>
{
    type Combined = SelectableTablesCons<T, CombinedSelectableTables<N, CombineWith>>;
}
