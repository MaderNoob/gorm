use std::{fmt::Write, marker::PhantomData};

use super::SqlStatement;
use crate::{
    sql::{
        Column, CombineSelectableTables, CombinedSelectableTables, FieldsConsListItem,
        HasForeignKey, ParameterBinder, SelectableTables, SelectedValues, SqlCondition,
        SqlExpression, Table, TableMarker,
    },
    TypedBool, TypedFalse, TypedTrue,
};

pub trait SelectStatement: SqlStatement {
    type OutputFields: FieldsConsListItem;
    type SelectFrom: SelectFrom;

    type HasSelectedValues: TypedBool;
    type HasWhereClause: TypedBool;
    type HasGroupByClause: TypedBool;
    type HasOrderByClause: TypedBool;

    fn write_selected_values<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    fn write_group_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    fn write_order_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;
}

impl<T: SelectStatement> SqlStatement for T {
    type OutputFields = <Self as SelectStatement>::OutputFields;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "SELECT ")?;
        self.write_selected_values(f, parameter_binder)?;
        write!(f, " FROM ")?;
        <Self as SelectStatement>::SelectFrom::write_sql_from_string(f)?;
        self.write_where_clause(f, parameter_binder)?;
        self.write_group_by_clause(f, parameter_binder)?;
        self.write_order_by_clause(f, parameter_binder)
    }
}

/// An sql statement for finding records in a table
pub struct EmptySelectStatement<S: SelectFrom>(PhantomData<S>);
impl<S: SelectFrom> EmptySelectStatement<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
impl<S: SelectFrom + 'static> SelectStatement for EmptySelectStatement<S> {
    type HasGroupByClause = TypedFalse;
    type HasOrderByClause = TypedFalse;
    type HasSelectedValues = TypedFalse;
    type HasWhereClause = TypedFalse;
    type OutputFields = <S::LeftMostTable as Table>::Fields;
    type SelectFrom = S;

    fn write_selected_values<'s, 'a>(
        &'s self,
        f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "*")
    }

    fn write_where_clause<'s, 'a>(
        &'s self,
        _f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        Ok(())
    }

    fn write_group_by_clause<'s, 'a>(
        &'s self,
        _f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        Ok(())
    }

    fn write_order_by_clause<'s, 'a>(
        &'s self,
        _f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        Ok(())
    }
}

pub struct WithSelectedValues<
    S: SelectFrom,
    T: SelectStatement<HasSelectedValues = TypedFalse>,
    V: SelectedValues<S::SelectableTables>,
> {
    statement: T,
    values: V,
    phantom: PhantomData<S>,
}
impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasSelectedValues = TypedFalse>,
    V: SelectedValues<S::SelectableTables> + 'static,
> SelectStatement for WithSelectedValues<S, T, V>
{
    type HasGroupByClause = T::HasGroupByClause;
    type HasOrderByClause = T::HasOrderByClause;
    type HasSelectedValues = TypedTrue;
    type HasWhereClause = T::HasWhereClause;
    type OutputFields = V::Fields;
    type SelectFrom = S;

    fn write_selected_values<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.values.write_sql_string(f, parameter_binder)
    }

    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_where_clause(f, parameter_binder)
    }

    fn write_group_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_group_by_clause(f, parameter_binder)
    }

    fn write_order_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_order_by_clause(f, parameter_binder)
    }
}

pub trait SelectValues: SelectStatement<HasSelectedValues = TypedFalse> {
    fn select<V: SelectedValues<<Self::SelectFrom as SelectFrom>::SelectableTables>>(
        self,
        values: V,
    ) -> WithSelectedValues<Self::SelectFrom, Self, V> {
        WithSelectedValues {
            statement: self,
            values,
            phantom: PhantomData,
        }
    }
}
impl<T: SelectStatement<HasSelectedValues = TypedFalse>> SelectValues for T {}

pub struct WithWhereClause<
    S: SelectFrom,
    T: SelectStatement<HasWhereClause = TypedFalse>,
    C: SqlCondition<S::SelectableTables>,
> {
    statement: T,
    condition: C,
    phantom: PhantomData<S>,
}
impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasWhereClause = TypedFalse>,
    C: SqlCondition<S::SelectableTables> + 'static,
> SelectStatement for WithWhereClause<S, T, C>
{
    type HasGroupByClause = T::HasGroupByClause;
    type HasOrderByClause = T::HasOrderByClause;
    type HasSelectedValues = T::HasSelectedValues;
    type HasWhereClause = TypedTrue;
    type OutputFields = <T as SelectStatement>::OutputFields;
    type SelectFrom = S;

    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, " WHERE ")?;
        self.condition.write_sql_string(f, parameter_binder)
    }

    fn write_selected_values<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_selected_values(f, parameter_binder)
    }

    fn write_group_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_group_by_clause(f, parameter_binder)
    }

    fn write_order_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_order_by_clause(f, parameter_binder)
    }
}

pub trait Filter: SelectStatement<HasWhereClause = TypedFalse> {
    fn filter<C: SqlCondition<<Self::SelectFrom as SelectFrom>::SelectableTables>>(
        self,
        condition: C,
    ) -> WithWhereClause<Self::SelectFrom, Self, C> {
        WithWhereClause {
            statement: self,
            condition,
            phantom: PhantomData,
        }
    }
}
impl<T: SelectStatement<HasWhereClause = TypedFalse>> Filter for T {}

pub struct WithGroupByClause<
    S: SelectFrom,
    T: SelectStatement<HasGroupByClause = TypedFalse>,
    G: SqlExpression<S::SelectableTables>,
> {
    statement: T,
    group_by: G,
    phantom: PhantomData<S>,
}
impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasGroupByClause = TypedFalse>,
    G: SqlExpression<S::SelectableTables> + 'static,
> SelectStatement for WithGroupByClause<S, T, G>
{
    type HasGroupByClause = TypedTrue;
    type HasOrderByClause = T::HasOrderByClause;
    type HasSelectedValues = T::HasSelectedValues;
    type HasWhereClause = T::HasWhereClause;
    type OutputFields = <T as SelectStatement>::OutputFields;
    type SelectFrom = S;

    fn write_group_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, " GROUP BY ")?;
        self.group_by.write_sql_string(f, parameter_binder)
    }

    fn write_selected_values<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_selected_values(f, parameter_binder)
    }

    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_where_clause(f, parameter_binder)
    }

    fn write_order_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_order_by_clause(f, parameter_binder)
    }
}

pub trait GroupBy: SelectStatement<HasGroupByClause = TypedFalse> {
    fn group_by<G: SqlExpression<<Self::SelectFrom as SelectFrom>::SelectableTables>>(
        self,
        group_by: G,
    ) -> WithGroupByClause<Self::SelectFrom, Self, G> {
        WithGroupByClause {
            statement: self,
            group_by,
            phantom: PhantomData,
        }
    }
}
impl<T: SelectStatement<HasGroupByClause = TypedFalse>> GroupBy for T {}

pub trait Order {
    const ORDER_STR: &'static str;
}
pub struct AscendingOrder;
impl Order for AscendingOrder {
    const ORDER_STR: &'static str = "";
}
pub struct DescendingOrder;
impl Order for DescendingOrder {
    const ORDER_STR: &'static str = " DESC";
}

pub struct WithOrderByClause<
    S: SelectFrom,
    T: SelectStatement<HasOrderByClause = TypedFalse>,
    B: SqlExpression<S::SelectableTables>,
    O: Order,
> {
    statement: T,
    order_by: B,
    phantom: (PhantomData<S>, PhantomData<O>),
}
impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasOrderByClause = TypedFalse>,
    B: SqlExpression<S::SelectableTables> + 'static,
    O: Order + 'static,
> SelectStatement for WithOrderByClause<S, T, B, O>
{
    type HasGroupByClause = T::HasGroupByClause;
    type HasOrderByClause = TypedTrue;
    type HasSelectedValues = T::HasSelectedValues;
    type HasWhereClause = T::HasWhereClause;
    type OutputFields = <T as SelectStatement>::OutputFields;
    type SelectFrom = S;

    fn write_group_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_group_by_clause(f, parameter_binder)
    }

    fn write_selected_values<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_selected_values(f, parameter_binder)
    }

    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_where_clause(f, parameter_binder)
    }

    fn write_order_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, " ORDER BY ")?;
        self.order_by.write_sql_string(f, parameter_binder)?;
        write!(f, "{}", O::ORDER_STR)
    }
}

pub trait OrderBy: SelectStatement<HasOrderByClause = TypedFalse> {
    fn order_by_ascending<B: SqlExpression<<Self::SelectFrom as SelectFrom>::SelectableTables>>(
        self,
        order_by: B,
    ) -> WithOrderByClause<Self::SelectFrom, Self, B, AscendingOrder> {
        WithOrderByClause {
            statement: self,
            order_by,
            phantom: (PhantomData, PhantomData),
        }
    }

    fn order_by_descending<B: SqlExpression<<Self::SelectFrom as SelectFrom>::SelectableTables>>(
        self,
        order_by: B,
    ) -> WithOrderByClause<Self::SelectFrom, Self, B, DescendingOrder> {
        WithOrderByClause {
            statement: self,
            order_by,
            phantom: (PhantomData, PhantomData),
        }
    }
}
impl<T: SelectStatement<HasOrderByClause = TypedFalse>> OrderBy for T {}

/// Something which you can select from.
/// This can be a table or multiple joined tables.
pub trait SelectFrom: Sized {
    type SelectableTables: SelectableTables;

    type LeftMostTable: Table;

    /// Writes the `from` part of the sql query as an sql string.
    fn write_sql_from_string(f: &mut String) -> std::fmt::Result;

    /// Writes the `from` part of the sql query without its left part an sql
    /// string. For example for `T: Table` this will write an empty string
    /// (`""`),  and for `InnerJoin<T1: Table, T2: Table>` this will write
    /// `"INNER JOIN T2 ON .."`.
    fn write_sql_from_string_without_left(f: &mut String) -> std::fmt::Result;

    /// Creates a select statement which finds records in this source.
    fn find(self) -> EmptySelectStatement<Self> {
        EmptySelectStatement::new()
    }
}

impl<T: TableMarker> SelectFrom for T {
    type LeftMostTable = T::Table;
    type SelectableTables = T::Table;

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
    type LeftMostTable = A::LeftMostTable;
    type SelectableTables = CombinedSelectableTables<A::SelectableTables, B::SelectableTables>;

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
