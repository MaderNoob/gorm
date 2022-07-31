use std::fmt::Write;
use std::marker::PhantomData;

use super::SqlStatement;
use crate::{
    bound_parameters::ParameterBinder,
    condition::SqlCondition,
    selectable_tables::{CombineSelectableTables, CombinedSelectableTables, SelectableTables},
    selected_values::SelectedValues,
    table::{Column, HasForeignKey, TableMarker},
    Table,
};

/// An sql statement for finding records in a table
pub struct SelectStatement<S: SelectFrom>(PhantomData<S>);
impl<S: SelectFrom> SelectStatement<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn select<V: SelectedValues<S::SelectableTables>>(
        self,
        selected_values: V,
    ) -> SelectStatementCustomValues<S, V> {
        SelectStatementCustomValues {
            values: selected_values,
            phantom: PhantomData,
        }
    }

    pub fn filter<C: SqlCondition<<S as SelectFrom>::SelectableTables>>(
        self,
        condition: C,
    ) -> FilteredSelectStatement<S, C> {
        FilteredSelectStatement {
            condition,
            phantom: PhantomData,
        }
    }
}
impl<S: SelectFrom + 'static> SqlStatement for SelectStatement<S> {
    type OutputFields = <S::LeftMostTable as Table>::Fields;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "SELECT * FROM ")?;
        S::write_sql_from_string(f)?;
        Ok(())
    }
}

/// An sql statement for finding records in a table which selects custom values
pub struct SelectStatementCustomValues<S: SelectFrom, V: SelectedValues<S::SelectableTables>> {
    values: V,
    phantom: PhantomData<S>,
}

impl<S: SelectFrom + 'static, V: SelectedValues<S::SelectableTables> + 'static> SqlStatement
    for SelectStatementCustomValues<S, V>
{
    type OutputFields = V::Fields;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "SELECT ")?;
        self.values.write_sql_string(f, parameter_binder)?;
        write!(f, " FROM ")?;
        S::write_sql_from_string(f)?;
        Ok(())
    }
}

/// An sql statement for finding records in a table matching some condition
pub struct FilteredSelectStatement<
    S: SelectFrom,
    C: SqlCondition<<S as SelectFrom>::SelectableTables>,
> {
    condition: C,
    phantom: PhantomData<S>,
}

impl<S: SelectFrom + 'static, C: SqlCondition<S::SelectableTables> + 'static> SqlStatement
    for FilteredSelectStatement<S, C>
{
    type OutputFields = <S::LeftMostTable as Table>::Fields;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "SELECT * FROM ")?;
        S::write_sql_from_string(f)?;
        write!(f, " where ")?;
        self.condition.write_sql_string(f, parameter_binder)
    }
}

/// Something which you can select from.
/// This can be a table or multiple joined tables.
pub trait SelectFrom: Sized {
    type SelectableTables: SelectableTables;

    type LeftMostTable: Table;

    /// Writes the `from` part of the sql query as an sql string.
    fn write_sql_from_string(f: &mut String) -> std::fmt::Result;

    /// Writes the `from` part of the sql query without its left part an sql string.
    /// For example for `T: Table` this will write an empty string (`""`),
    ///  and for `InnerJoin<T1: Table, T2: Table>` this will write `"INNER JOIN T2 ON .."`.
    fn write_sql_from_string_without_left(f: &mut String) -> std::fmt::Result;

    /// Creates a select statement which finds records in this source.
    fn find(self) -> SelectStatement<Self> {
        SelectStatement::new()
    }
}

impl<T: TableMarker> SelectFrom for T {
    type SelectableTables = T::Table;

    type LeftMostTable = T::Table;

    fn write_sql_from_string(f: &mut String) -> std::fmt::Result {
        write!(f, "\"{}\"", T::Table::TABLE_NAME)
    }

    fn write_sql_from_string_without_left(_f: &mut String) -> std::fmt::Result {
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
    A::LeftMostTable: HasForeignKey<B::LeftMostTable>,
{
    type SelectableTables = CombinedSelectableTables<A::SelectableTables, B::SelectableTables>;

    type LeftMostTable = A::LeftMostTable;

    fn write_sql_from_string(f: &mut String) -> std::fmt::Result {
        A::write_sql_from_string(f)?;
        Self::write_sql_from_string_without_left(f)
    }

    fn write_sql_from_string_without_left(f: &mut String) -> std::fmt::Result {
        write!(
            f,
            " INNER JOIN \"{}\" ON \"{}\".\"{}\" = \"{}\".\"id\"",
            B::LeftMostTable::TABLE_NAME,
            A::LeftMostTable::TABLE_NAME,
            <<A::LeftMostTable as HasForeignKey<B::LeftMostTable>>::ForeignKeyColumn as Column>::COLUMN_NAME,
            B::LeftMostTable::TABLE_NAME
        )?;

        B::write_sql_from_string_without_left(f)
    }
}

pub trait InnerJoinTrait: Sized + SelectFrom {
    /// Inner joins this table with another table
    fn inner_join<S: SelectFrom>(self, _with: S) -> InnerJoined<Self, S>
    where
        Self::LeftMostTable: HasForeignKey<S::LeftMostTable>,
    {
        InnerJoined::new()
    }
}

impl<S: SelectFrom> InnerJoinTrait for S {}
