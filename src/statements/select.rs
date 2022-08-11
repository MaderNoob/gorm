use std::{fmt::Write, marker::PhantomData};

use super::SqlStatement;
use crate::{
    sql::{
        Column, ColumnIsForeignKey, CombineSelectableTables, CombinedSelectableTables,
        FieldNameCharsConsListItem, FieldsConsListItem, ParameterBinder, SelectableTables,
        SelectedValues, SelectedValuesContainsFieldWithName, SqlBool, SqlExpression, SqlType,
        Table, TableHasOneForeignKey, TableMarker,
    },
    util::{TypedBool, TypedFalse, TypedTrue, TypesEqual},
};

/// Represents any type of sql select statement.
pub trait SelectStatement: SqlStatement {
    /// A type identifying the output fields of this select statement.
    type OutputFields: FieldsConsListItem;

    /// The source from which records are being selected.
    type SelectFrom: SelectFrom;

    /// The selected values in this select statement.
    ///
    /// For select statement which don't have custom selected values, this will
    /// be `()`, otherwise it will be some type implementing the
    /// [`SelectedValues`] trait.
    // This is used for allowing ordering by a value that was selected using
    // `select_values!`. In order to check that the value is actually in
    // the list of selected values we need a reference to the
    // `SelectedValues` because it implements the
    // `SelectedValuesContainsFieldWithName` trait which allows checking if it
    // contains a field with a given name.
    //
    // We can't put a constraint on this type to implement `SelectedValues`
    // because that would require adding a generic to the `SelectStatement`
    // trait, which will break some other stuff. Also, there won't be an
    // easy placeholder to use if we constrained it.
    //
    // So we just don't constrain it, and for types that don't have custom
    // values it will be `()` or whatever else, and for types that have
    // custom values it would implement `SelectedValues`, that way we could
    // implement something only when this implements `SelectedValues`.
    type SelectedValues;

    /// Does this select statement have custom selected values?
    type HasSelectedValues: TypedBool;

    /// Does this select statement have a `WHERE` clause?
    type HasWhereClause: TypedBool;

    /// Does this select statement have a `GROUP BY` clause?
    type HasGroupByClause: TypedBool;

    /// Does this select statement have a `ORDER BY` clause?
    type HasOrderByClause: TypedBool;

    /// Writes the custom selected values which are selected by this select
    /// statement.
    fn write_selected_values<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes the `WHERE` clause of this select statement.
    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes the `GROUP BY` clause of this select statement.
    fn write_group_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes the `ORDER BY` clause of this select statement.
    fn write_order_by_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes this select statement as an sql string.
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

/// Implements the [`SqlStatement`] trait for some type which implements
/// [`SelectStatement`]
macro_rules! impl_sql_statement_for_select_statement {
    {} => {
        type OutputFields = <Self as SelectStatement>::OutputFields;

        fn write_sql_string<'s, 'a>(
            &'s self,
            f: &mut String,
            parameter_binder: &mut ParameterBinder<'a>,
        ) -> std::fmt::Result
        where
            's: 'a,
        {
            <Self as SelectStatement>::write_sql_string(&self, f, parameter_binder)
        }
    };
}

/// An sql select statement which loads all records from some table.
///
/// This statement can be created by calling the [`SelectFrom::find`] function.
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
    type SelectedValues = ();

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

impl<S: SelectFrom + 'static> SqlStatement for EmptySelectStatement<S> {
    impl_sql_statement_for_select_statement! {}
}

/// A wrapper around an sql select statement which selects custom values from
/// it. for it.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`SelectValues::select`] function.
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
    type SelectedValues = V;

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

impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasSelectedValues = TypedFalse>,
    V: SelectedValues<S::SelectableTables> + 'static,
> SqlStatement for WithSelectedValues<S, T, V>
{
    impl_sql_statement_for_select_statement! {}
}

/// A trait which allows selecting custom values from an sql select statement.
pub trait SelectValues: SelectStatement<HasSelectedValues = TypedFalse> {
    /// Selects custom values from this select statement. To provide a list of
    /// selected values use the [`select_values!`] macro.
    ///
    /// [`select_values!`]: crate::select_values
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

/// A wrapper around an sql select statement which adds a `WHERE` clause to it.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`Filter::filter`] function.
pub struct WithWhereClause<
    S: SelectFrom,
    T: SelectStatement<HasWhereClause = TypedFalse>,
    C: SqlExpression<S::SelectableTables, SqlType = SqlBool>,
> {
    statement: T,
    condition: C,
    phantom: PhantomData<S>,
}
impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasWhereClause = TypedFalse>,
    C: SqlExpression<S::SelectableTables, SqlType = SqlBool> + 'static,
> SelectStatement for WithWhereClause<S, T, C>
{
    type HasGroupByClause = T::HasGroupByClause;
    type HasOrderByClause = T::HasOrderByClause;
    type HasSelectedValues = T::HasSelectedValues;
    type HasWhereClause = TypedTrue;
    type OutputFields = <T as SelectStatement>::OutputFields;
    type SelectFrom = S;
    type SelectedValues = T::SelectedValues;

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

impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasWhereClause = TypedFalse>,
    C: SqlExpression<S::SelectableTables, SqlType = SqlBool> + 'static,
> SqlStatement for WithWhereClause<S, T, C>
{
    impl_sql_statement_for_select_statement! {}
}

/// A trait which allows filtering a select statement so that it only selects
/// records matching some condition.
pub trait Filter: SelectStatement<HasWhereClause = TypedFalse> {
    /// Filters this select statement, so that it only returns records which
    /// match the given condition.
    fn filter<
        C: SqlExpression<<Self::SelectFrom as SelectFrom>::SelectableTables, SqlType = SqlBool>,
    >(
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

/// A wrapper around an sql select statement which adds a `GROUP BY` clause to
/// it.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`GroupBy::group_by`] function.
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
    type SelectedValues = T::SelectedValues;

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

impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasGroupByClause = TypedFalse>,
    G: SqlExpression<S::SelectableTables> + 'static,
> SqlStatement for WithGroupByClause<S, T, G>
{
    impl_sql_statement_for_select_statement! {}
}

/// A trait which allows grouping the results of a select statement which uses
/// aggregate functions by some expression.
pub trait GroupBy: SelectStatement<HasGroupByClause = TypedFalse> {
    /// Groups the results of this select statement which uses aggregate
    /// functions by the given expression.
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

/// An ordering, used in an `ORDER BY` clause.
pub trait Ordering {
    /// The sql string which should be appended at the end of the `ORDER BY`
    /// clause for this ordering to be applied.
    const ORDER_STR: &'static str;
}

/// Order the results of a query in ascending order.
pub struct AscendingOrder;
impl Ordering for AscendingOrder {
    const ORDER_STR: &'static str = "";
}

/// Order the results of a query in descending order.
pub struct DescendingOrder;
impl Ordering for DescendingOrder {
    const ORDER_STR: &'static str = " DESC";
}

/// A wrapper around an sql select statement which adds an `ORDER BY` clause to
/// it and orders it by some expression.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`OrderBy::order_by_ascending`] and [`OrderBy::order_by_descending`]
/// functions.
pub struct WithOrderByClause<
    S: SelectFrom,
    T: SelectStatement<HasOrderByClause = TypedFalse>,
    B: SqlExpression<S::SelectableTables>,
    O: Ordering,
> {
    statement: T,
    order_by: B,
    _phantom: (PhantomData<S>, PhantomData<O>),
}
impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasOrderByClause = TypedFalse>,
    B: SqlExpression<S::SelectableTables> + 'static,
    O: Ordering + 'static,
> SelectStatement for WithOrderByClause<S, T, B, O>
{
    type HasGroupByClause = T::HasGroupByClause;
    type HasOrderByClause = TypedTrue;
    type HasSelectedValues = T::HasSelectedValues;
    type HasWhereClause = T::HasWhereClause;
    type OutputFields = <T as SelectStatement>::OutputFields;
    type SelectFrom = S;
    type SelectedValues = T::SelectedValues;

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

impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasOrderByClause = TypedFalse>,
    B: SqlExpression<S::SelectableTables> + 'static,
    O: Ordering + 'static,
> SqlStatement for WithOrderByClause<S, T, B, O>
{
    impl_sql_statement_for_select_statement! {}
}

/// A trait which allows ordering the results of a select statement by some
/// expression in an ascending or descending order.
pub trait OrderBy: SelectStatement<HasOrderByClause = TypedFalse> {
    /// Orders the results of this select statement by the given expression in
    /// an ascending order.
    fn order_by_ascending<B: SqlExpression<<Self::SelectFrom as SelectFrom>::SelectableTables>>(
        self,
        order_by: B,
    ) -> WithOrderByClause<Self::SelectFrom, Self, B, AscendingOrder> {
        WithOrderByClause {
            statement: self,
            order_by,
            _phantom: (PhantomData, PhantomData),
        }
    }

    /// Orders the results of this select statement by the given expression in
    /// an descending order.
    fn order_by_descending<B: SqlExpression<<Self::SelectFrom as SelectFrom>::SelectableTables>>(
        self,
        order_by: B,
    ) -> WithOrderByClause<Self::SelectFrom, Self, B, DescendingOrder> {
        WithOrderByClause {
            statement: self,
            order_by,
            _phantom: (PhantomData, PhantomData),
        }
    }
}
impl<T: SelectStatement<HasOrderByClause = TypedFalse>> OrderBy for T {}

/// A selected value which is used to order by.
pub trait SelectedValueToOrderBy {
    /// A type used to identify the name of this selected value in the list of
    /// selected values.
    type Name: FieldNameCharsConsListItem;

    /// The name of this selected value as a string.
    const NAME_STR: &'static str;
}

/// A wrapper around an sql select statement which adds an `ORDER BY` clause to
/// it and orders it by some value which is in the list of custom selected
/// values.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`OrderBySelectedValue::order_by_selected_value_ascending`] and
/// [`OrderBySelectedValue::order_by_selected_value_descending`]
/// functions.
pub struct WithOrderBySelectedValueClause<
    S: SelectFrom,
    T: SelectStatement<HasOrderByClause = TypedFalse>,
    B: SelectedValueToOrderBy,
    O: Ordering,
> {
    statement: T,
    _order_by: B,
    _phantom: (PhantomData<S>, PhantomData<O>),
}
impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasOrderByClause = TypedFalse>,
    B: SelectedValueToOrderBy + 'static,
    O: Ordering + 'static,
> SelectStatement for WithOrderBySelectedValueClause<S, T, B, O>
{
    type HasGroupByClause = T::HasGroupByClause;
    type HasOrderByClause = TypedTrue;
    type HasSelectedValues = T::HasSelectedValues;
    type HasWhereClause = T::HasWhereClause;
    type OutputFields = <T as SelectStatement>::OutputFields;
    type SelectFrom = S;
    type SelectedValues = T::SelectedValues;

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
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, " ORDER BY {}{}", B::NAME_STR, O::ORDER_STR)
    }
}

impl<
    S: SelectFrom + 'static,
    T: SelectStatement<HasOrderByClause = TypedFalse>,
    B: SelectedValueToOrderBy + 'static,
    O: Ordering + 'static,
> SqlStatement for WithOrderBySelectedValueClause<S, T, B, O>
{
    impl_sql_statement_for_select_statement! {}
}

/// A trait which allows ordering the results of a select statement by some
/// value which is in the list of custom selected values of this statement in an
/// ascending or descending order.
pub trait OrderBySelectedValue<S: SelectableTables>:
    SelectStatement<HasOrderByClause = TypedFalse>
where
    Self::SelectedValues: SelectedValues<S>,
{
    /// Orders the results of this select statement in an ascending order by the
    /// given value from the list of custom selected values of this statment.
    /// To provide a value to this function you should use the
    /// [`selected_value_to_order_by!`] macro
    ///
    /// [`selected_value_to_order_by!`]: crate::selected_value_to_order_by
    fn order_by_selected_value_ascending<B: SelectedValueToOrderBy>(
        self,
        order_by: B,
    ) -> WithOrderBySelectedValueClause<Self::SelectFrom, Self, B, AscendingOrder>
    where
        Self::SelectedValues: SelectedValuesContainsFieldWithName<B::Name>,
    {
        WithOrderBySelectedValueClause {
            statement: self,
            _order_by: order_by,
            _phantom: (PhantomData, PhantomData),
        }
    }

    /// Orders the results of this select statement in an descending order by
    /// the given value from the list of custom selected values of this
    /// statment. To provide a value to this function you should use the
    /// [`selected_value_to_order_by!`] macro
    ///
    /// [`selected_value_to_order_by!`]: crate::selected_value_to_order_by
    fn order_by_selected_value_descending<B: SelectedValueToOrderBy>(
        self,
        order_by: B,
    ) -> WithOrderBySelectedValueClause<Self::SelectFrom, Self, B, DescendingOrder>
    where
        Self::SelectedValues: SelectedValuesContainsFieldWithName<B::Name>,
    {
        WithOrderBySelectedValueClause {
            statement: self,
            _order_by: order_by,
            _phantom: (PhantomData, PhantomData),
        }
    }
}
impl<S: SelectableTables, T: SelectStatement<HasOrderByClause = TypedFalse>> OrderBySelectedValue<S>
    for T
where
    T::SelectedValues: SelectedValues<S>,
{
}

/// Some source which you can select from.
///
/// This can be a table or multiple tables joined together.
pub trait SelectFrom: Sized {
    /// The tables from which columns can be selected in a statement which
    /// selects values from this source.
    type SelectableTables: SelectableTables;

    /// The left-most table of this source.
    /// This is the table whose values will be selected by default in cause of
    /// inner joined tables.
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

// We can select records from a table.
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

/// Represents the inner join of 2 selection sources.
pub struct InnerJoined<
    A: SelectFrom,
    B: SelectFrom,
    C: Column<Table = A::LeftMostTable> + ColumnIsForeignKey<B::LeftMostTable>,
>(PhantomData<A>, PhantomData<B>, PhantomData<C>)
where
    A::SelectableTables: CombineSelectableTables<B::SelectableTables>,
    (
        <<C as Column>::SqlType as SqlType>::NonNullSqlType,
        <<B::LeftMostTable as Table>::IdColumn as Column>::SqlType,
    ): TypesEqual;

impl<
    A: SelectFrom,
    B: SelectFrom,
    C: Column<Table = A::LeftMostTable> + ColumnIsForeignKey<B::LeftMostTable>,
> InnerJoined<A, B, C>
where
    A::SelectableTables: CombineSelectableTables<B::SelectableTables>,
    (
        <<C as Column>::SqlType as SqlType>::NonNullSqlType,
        <<B::LeftMostTable as Table>::IdColumn as Column>::SqlType,
    ): TypesEqual,
{
    /// Creates a new source which represents the inner join of 2 selection
    /// sources.
    pub fn new() -> Self {
        Self(PhantomData, PhantomData, PhantomData)
    }
}

// We can select from an inner joined source if there is a foreign key
// constraint using which we can join the 2 sources.
impl<
    A: SelectFrom,
    B: SelectFrom,
    C: Column<Table = A::LeftMostTable> + ColumnIsForeignKey<B::LeftMostTable>,
> SelectFrom for InnerJoined<A, B, C>
where
    A::SelectableTables: CombineSelectableTables<B::SelectableTables>,
    (
        <<C as Column>::SqlType as SqlType>::NonNullSqlType,
        <<B::LeftMostTable as Table>::IdColumn as Column>::SqlType,
    ): TypesEqual,
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
            <C as Column>::COLUMN_NAME,
            B::LeftMostTable::TABLE_NAME
        )?;

        B::write_sql_from_string_without_left(f)
    }
}

/// A trait which allows inner joining 2 selection sources using a foreign key.
pub trait InnerJoinTrait: Sized + SelectFrom {
    /// Inner joins this selection source with another selection source, if this
    /// source has a foreign key to the other one.
    fn inner_join<S: SelectFrom>(self, _with: S) -> InnerJoined<Self, S, <Self::LeftMostTable as TableHasOneForeignKey<S::LeftMostTable>>::ForeignKeyColumn>
    where
        Self::LeftMostTable: TableHasOneForeignKey<S::LeftMostTable>,
        <Self as SelectFrom>::SelectableTables: CombineSelectableTables<<S as SelectFrom>::SelectableTables>,
        (<<<Self::LeftMostTable as TableHasOneForeignKey<S::LeftMostTable>>::ForeignKeyColumn as Column>::SqlType as SqlType>::NonNullSqlType, <<S::LeftMostTable as Table>::IdColumn as Column>::SqlType): TypesEqual
    {
        InnerJoined::new()
    }
}

impl<S: SelectFrom> InnerJoinTrait for S {}

/// A trait which allows inner joining 2 selection sources using a foreign keys
/// on a specific column.
pub trait InnerJoinOnTrait: Sized + SelectFrom {
    /// Inner joins this selection source with another selection source, if this
    /// source has a foreign key to the other one.
    fn inner_join_on_column<
        S: SelectFrom,
        C: Column<Table = Self::LeftMostTable> + ColumnIsForeignKey<S::LeftMostTable>,
    >(
        self,
        _column: C,
        _with: S,
    ) -> InnerJoined<Self, S, C>
    where
        <Self as SelectFrom>::SelectableTables:
            CombineSelectableTables<<S as SelectFrom>::SelectableTables>,
        (
            <<C as Column>::SqlType as SqlType>::NonNullSqlType,
            <<S::LeftMostTable as Table>::IdColumn as Column>::SqlType,
        ): TypesEqual,
    {
        InnerJoined::new()
    }
}

impl<S: SelectFrom> InnerJoinOnTrait for S {}
