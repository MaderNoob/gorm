use std::{fmt::Write, marker::PhantomData};

use super::SqlStatement;
use crate::{
    sql::{FieldsConsListItem, ParameterBinder, SelectedValues, SqlBool, SqlExpression, UpdateSet},
    util::{TypedBool, TypedConsListNil, TypedFalse, TypedTrue},
    Table,
};

/// Represents any type of sql update statement.
pub trait UpdateStatement: Sized {
    /// A type identifying the output fields of this update statement, selected
    /// in its `RETURNING` clause.
    type OutputFields: FieldsConsListItem;

    /// The table that this statement updates.
    type UpdateTable: Table;

    /// Does this update statement have a `WHERE` clause?
    type HasWhereClause: TypedBool;

    /// Does this update statement have a `RETURNING` clause?
    type HasReturningClause: TypedBool;

    /// The update set which this update statement should perform on each row
    /// which matches the condition.
    type UpdateSet: UpdateSet;

    /// Writes the update set of this update statement.
    ///
    /// This is a list of comma seperated assignments to columns of the table.
    fn write_update_set<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes the `WHERE` clause of this update statement.
    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes the `RETURNING` clause of this update statement.
    fn write_returning_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes this update statement as an sql string.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(
            f,
            "UPDATE {} SET ",
            <Self::UpdateTable as Table>::TABLE_NAME
        )?;
        self.write_update_set(f, parameter_binder)?;
        self.write_where_clause(f, parameter_binder)?;
        self.write_returning_clause(f, parameter_binder)
    }
}

/// Implements the [`SqlStatement`] trait for some type which implements
/// [`UpdateStatement`]
macro_rules! impl_sql_statement_for_update_statement {
    {} => {
        type OutputFields = <Self as UpdateStatement>::OutputFields;

        fn write_sql_string<'s, 'a>(
            &'s self,
            f: &mut String,
            parameter_binder: &mut ParameterBinder<'a>,
        ) -> std::fmt::Result
        where
            's: 'a,
        {
            <Self as UpdateStatement>::write_sql_string(&self, f, parameter_binder)
        }
    };
}

/// An empty sql update statement which doesn't have an update set yet. You can
/// use the `set` function to add an update set to this statement.
///
/// This statement can be created by calling the [`TableMarker::update`]
/// function.
///
/// [`TableMarker::update`]: crate::sql::TableMarker::update
pub struct EmptyUpdateStatement<T: Table>(PhantomData<T>);
impl<T: Table> EmptyUpdateStatement<T> {
    /// Creates a new empty sql update statement which doesn't have an update
    /// set.
    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// Adds an update set to this update statement which defines which columns
    /// should be set and what their values should be.
    ///
    /// To create an update set for this function, use the [`update_set!`]
    /// macro.
    ///
    /// [`update_set!`]: gorm_macros::update_set
    pub fn set<U: UpdateSet<UpdateTable = T>>(
        self,
        update_set: U,
    ) -> UpdateStatementWithUpdateSet<U> {
        UpdateStatementWithUpdateSet::new(update_set)
    }
}

/// An sql update statement which updates all rows in the table according to its
/// update set.
///
/// This statement can be created by calling the [`EmptyUpdateStatement::set`]
/// function.
pub struct UpdateStatementWithUpdateSet<U: UpdateSet> {
    update_set: U,
}
impl<U: UpdateSet> UpdateStatementWithUpdateSet<U> {
    pub fn new(update_set: U) -> Self {
        Self { update_set }
    }
}

impl<U: UpdateSet> UpdateStatement for UpdateStatementWithUpdateSet<U> {
    type HasReturningClause = TypedFalse;
    type HasWhereClause = TypedFalse;
    type OutputFields = TypedConsListNil;
    type UpdateSet = U;
    type UpdateTable = U::UpdateTable;

    /// Writes the update set of this update statement.
    ///
    /// This is a list of comma seperated assignments to columns of the table.
    fn write_update_set<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.update_set.write_sql_string(f, parameter_binder)
    }

    /// Writes the `WHERE` clause of this update statement.
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

    /// Writes the `RETURNING` clause of this update statement.
    fn write_returning_clause<'s, 'a>(
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

impl<U: UpdateSet> SqlStatement for UpdateStatementWithUpdateSet<U> {
    impl_sql_statement_for_update_statement! {}
}

/// A wrapper around an sql update statement which adds a `WHERE` clause to it.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`FilterUpdateStatement::filter`] function.
pub struct UpdateWithWhereClause<
    S: UpdateStatement<HasWhereClause = TypedFalse>,
    C: SqlExpression<S::UpdateTable, SqlType = SqlBool>,
> {
    statement: S,
    condition: C,
}

impl<
    S: UpdateStatement<HasWhereClause = TypedFalse>,
    C: SqlExpression<S::UpdateTable, SqlType = SqlBool>,
> UpdateStatement for UpdateWithWhereClause<S, C>
{
    type HasReturningClause = S::HasReturningClause;
    type HasWhereClause = TypedTrue;
    type OutputFields = S::OutputFields;
    type UpdateSet = S::UpdateSet;
    type UpdateTable = S::UpdateTable;

    /// Writes the update set of this update statement.
    ///
    /// This is a list of comma seperated assignments to columns of the table.
    fn write_update_set<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_update_set(f, parameter_binder)
    }

    /// Writes the `WHERE` clause of this update statement.
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

    /// Writes the `RETURNING` clause of this update statement.
    fn write_returning_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_returning_clause(f, parameter_binder)
    }
}

impl<
    S: UpdateStatement<HasWhereClause = TypedFalse> + 'static,
    C: SqlExpression<S::UpdateTable, SqlType = SqlBool> + 'static,
> SqlStatement for UpdateWithWhereClause<S, C>
{
    impl_sql_statement_for_update_statement! {}
}

/// A trait which allows filtering an update statement so that it only updates
/// records matching some condition.
pub trait FilterUpdateStatement: UpdateStatement<HasWhereClause = TypedFalse> {
    /// Filters this update statement, so that it only updates records which
    /// match the given condition.
    fn filter<C: SqlExpression<Self::UpdateTable, SqlType = SqlBool>>(
        self,
        condition: C,
    ) -> UpdateWithWhereClause<Self, C> {
        UpdateWithWhereClause {
            statement: self,
            condition,
        }
    }
}

impl<T: UpdateStatement<HasWhereClause = TypedFalse>> FilterUpdateStatement for T {}

/// A wrapper around an sql update statement which adds a `RETURNING` clause to
/// it.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`UpdateStatementReturning::returning`] function.
pub struct UpdateWithReturningClause<
    S: UpdateStatement<HasReturningClause = TypedFalse>,
    R: SelectedValues<S::UpdateTable>,
> {
    statement: S,
    returning: R,
}

impl<S: UpdateStatement<HasReturningClause = TypedFalse>, R: SelectedValues<S::UpdateTable>>
    UpdateStatement for UpdateWithReturningClause<S, R>
{
    type HasReturningClause = TypedTrue;
    type HasWhereClause = S::HasWhereClause;
    type OutputFields = R::Fields;
    type UpdateSet = S::UpdateSet;
    type UpdateTable = S::UpdateTable;

    /// Writes the update set of this update statement.
    ///
    /// This is a list of comma seperated assignments to columns of the table.
    fn write_update_set<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_update_set(f, parameter_binder)
    }

    /// Writes the `WHERE` clause of this update statement.
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

    /// Writes the `RETURNING` clause of this update statement.
    fn write_returning_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, " RETURNING ")?;
        self.returning.write_sql_string(f, parameter_binder)
    }
}

impl<S: UpdateStatement<HasReturningClause = TypedFalse>, R: SelectedValues<S::UpdateTable>>
    SqlStatement for UpdateWithReturningClause<S, R>
{
    impl_sql_statement_for_update_statement! {}
}

/// A trait which allows returning some values from the records updated by some
/// update statement.
pub trait UpdateStatementReturning: UpdateStatement<HasReturningClause = TypedFalse> {
    /// Selects some values to be returned from the records updates by this
    /// updates statement. To provide a list of values to be returned, use the
    /// [`returning!`] macro.
    ///
    /// [`returning!`]: crate::returning
    fn returning<R: SelectedValues<Self::UpdateTable>>(
        self,
        returning: R,
    ) -> UpdateWithReturningClause<Self, R> {
        UpdateWithReturningClause {
            statement: self,
            returning,
        }
    }
}

impl<T: UpdateStatement<HasReturningClause = TypedFalse>> UpdateStatementReturning for T {}
