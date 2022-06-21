use std::{fmt::Display, marker::PhantomData};

use crate::{
    selectable_tables::{CombineSelectableTables, CombinedSelectableTables, SelectableTables},
    table::{Table, TableMarker},
};

use super::SelectStatement;

/// Something which you can select from.
/// This can be a table or multiple joined tables.
pub trait SelectFrom: Sized {
    type SelectableTables: SelectableTables;

    type LeftMostTable: Table;

    /// Writes the `from` part of the sql query as an sql string.
    fn write_sql_from_string(f: &mut std::fmt::Formatter) -> std::fmt::Result;

    /// Writes the `from` part of the sql query without its left part an sql string.
    /// For example for `T: Table` this will write an empty string (`""`),
    ///  and for `InnerJoin<T1: Table, T2: Table>` this will write `"INNER JOIN T2 ON .."`.
    fn write_sql_from_string_without_left(f: &mut std::fmt::Formatter) -> std::fmt::Result;

    /// Creates a select statement which finds records in this source.
    fn find(self) -> SelectStatement<Self> {
        SelectStatement::new()
    }
}

impl<T: TableMarker> SelectFrom for T {
    type SelectableTables = T::Table;

    type LeftMostTable = T::Table;

    fn write_sql_from_string(f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\"{}\"", T::Table::TABLE_NAME)
    }

    fn write_sql_from_string_without_left(_f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }
}

pub struct InnerJoined<A: SelectFrom, B: SelectFrom>(PhantomData<A>, PhantomData<B>);
impl<A: SelectFrom, B: SelectFrom> InnerJoined<A, B> {
    pub fn new() -> Self {
        Self(PhantomData, PhantomData)
    }
}
impl<A: SelectFrom, B: SelectFrom> SelectFrom for InnerJoined<A, B>
where
    A::SelectableTables: CombineSelectableTables<B::SelectableTables>,
{
    type SelectableTables = CombinedSelectableTables<A::SelectableTables, B::SelectableTables>;

    type LeftMostTable = A::LeftMostTable;

    fn write_sql_from_string(f: &mut std::fmt::Formatter) -> std::fmt::Result {
        A::write_sql_from_string(f)?;
        Self::write_sql_from_string_without_left(f)
    }

    fn write_sql_from_string_without_left(f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            " inner join \"{}\" on condition_for({},{})",
            B::LeftMostTable::TABLE_NAME,
            Self::LeftMostTable::TABLE_NAME,
            B::LeftMostTable::TABLE_NAME
        )?;

        B::write_sql_from_string_without_left(f)
    }
}

/// A formatter which formats the from part of an sql query for some `SelectFrom`.
pub struct SelectFromFormatter<S: SelectFrom>(PhantomData<S>);
impl<S: SelectFrom> SelectFromFormatter<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
impl<S: SelectFrom> Display for SelectFromFormatter<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        S::write_sql_from_string(f)
    }
}

pub trait InnerJoinTrait: Sized + SelectFrom {
    /// Inner joins this table with another table
    fn inner_join<S: SelectFrom>(self, _with: S) -> InnerJoined<Self, S> {
        InnerJoined::new()
    }
}

impl<S: SelectFrom> InnerJoinTrait for S {}
