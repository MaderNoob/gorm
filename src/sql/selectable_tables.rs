use std::marker::PhantomData;

use crate::{sql::Table, util::TypesNotEqual};

/// A typed cons list of tables from which columns can be used in an sql
/// expression.
pub trait SelectableTables {}

/// A marker trait which indicates that a list of selectable tables contains
/// some table.
pub trait SelectableTablesContains<T: Table>: SelectableTables {}

/// A cons item in the selectable tables cons list.
pub struct SelectableTablesCons<T: Table, N: SelectableTables>(PhantomData<T>, PhantomData<N>);

// A table is a `SelectableTables`, and as such, it contains itself
impl<T: Table> SelectableTables for T {}
impl<T: Table> SelectableTablesContains<T> for T {}

// a cons item is a `SelectableTables`
impl<T: Table, N: SelectableTables> SelectableTables for SelectableTablesCons<T, N> {}

// a cons list item contains the Table it holds, and everything else that its
// next holds which is not the same as the table it holds.
impl<T: Table, N: SelectableTables> SelectableTablesContains<T> for SelectableTablesCons<T, N> {}
impl<T: Table, N: SelectableTables, InnerT: Table> SelectableTablesContains<InnerT>
    for SelectableTablesCons<T, N>
where
    N: SelectableTablesContains<InnerT>,
    (T, InnerT): TypesNotEqual,
{
}

/// A trait used to combine 2 selectable tables types.
pub trait CombineSelectableTables<CombineWith: SelectableTables> {
    type Combined: SelectableTables;
}

impl<T: Table + SelectableTables, CombineWith: SelectableTables>
    CombineSelectableTables<CombineWith> for T
{
    type Combined = SelectableTablesCons<T, CombineWith>;
}

/// A type alias which represents the result of combining 2 types which
/// implement the [`SelectableTables`] trait into a single type which implements
/// that trait.
pub type CombinedSelectableTables<A, B> = <A as CombineSelectableTables<B>>::Combined;

impl<
    T: Table,
    N: SelectableTables + CombineSelectableTables<CombineWith>,
    CombineWith: SelectableTables,
> CombineSelectableTables<CombineWith> for SelectableTablesCons<T, N>
{
    type Combined = SelectableTablesCons<T, CombinedSelectableTables<N, CombineWith>>;
}
