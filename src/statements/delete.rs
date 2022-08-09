use std::marker::PhantomData;

use super::SqlStatement;
use crate::{
    sql::{FieldsConsListItem, ParameterBinder, SelectedValues, SqlBool, SqlExpression},
    util::{TypedBool, TypedConsListNil, TypedFalse, TypedTrue},
    Table,
};

/// Represents any type of sql delete statement.
pub trait DeleteStatement: Sized {
    /// A type identifying the output fields of this delete statement, selected
    /// in its `RETURNING` clause.
    type OutputFields: FieldsConsListItem;

    /// The table that this statement deletes from.
    type DeleteFrom: Table;

    /// Does this delete statement have a `WHERE` clause?
    type HasWhereClause: TypedBool;

    /// Does this delete statement have a `RETURNING` clause?
    type HasReturningClause: TypedBool;

    /// Writes the `WHERE` clause of this delete statement.
    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes the `RETURNING` clause of this delete statement.
    fn write_returning_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes this delete statement as an sql string.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        use std::fmt::Write;

        write!(
            f,
            "DELETE  FROM {}",
            <Self::DeleteFrom as Table>::TABLE_NAME
        )?;
        self.write_where_clause(f, parameter_binder)?;
        self.write_returning_clause(f, parameter_binder)
    }
}

/// Implements the [`SqlStatement`] trait for some type which implements
/// [`DeleteStatement`]
macro_rules! impl_sql_statement_for_delete_statement {
    {} => {
        type OutputFields = <Self as DeleteStatement>::OutputFields;

        fn write_sql_string<'s, 'a>(
            &'s self,
            f: &mut String,
            parameter_binder: &mut ParameterBinder<'a>,
        ) -> std::fmt::Result
        where
            's: 'a,
        {
            <Self as DeleteStatement>::write_sql_string(&self, f, parameter_binder)
        }
    };
}

/// An sql delete statement which deletes all records from the table.
///
/// This statement can be created by calling the [`TableMarker::delete`]
/// function.
///
/// [`TableMarker::delete`]: crate::sql::TableMarker::delete
pub struct EmptyDeleteStatement<T: Table>(PhantomData<T>);
impl<T: Table> EmptyDeleteStatement<T> {
    /// Creates a new sql delete statement which deletes all records from the
    /// table
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Table> DeleteStatement for EmptyDeleteStatement<T> {
    type DeleteFrom = T;
    type HasReturningClause = TypedFalse;
    type HasWhereClause = TypedFalse;
    type OutputFields = TypedConsListNil;

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

impl<T: Table> SqlStatement for EmptyDeleteStatement<T> {
    impl_sql_statement_for_delete_statement! {}
}

/// A wrapper around an sql delete statement which adds a `WHERE` clause to it.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`FilterDeleteStatement::filter`] function.
pub struct DeleteWithWhereClause<
    S: DeleteStatement<HasWhereClause = TypedFalse>,
    C: SqlExpression<S::DeleteFrom, SqlType = SqlBool>,
> {
    statement: S,
    condition: C,
}

impl<
        S: DeleteStatement<HasWhereClause = TypedFalse>,
        C: SqlExpression<S::DeleteFrom, SqlType = SqlBool>,
    > DeleteStatement for DeleteWithWhereClause<S, C>
{
    type DeleteFrom = S::DeleteFrom;
    type HasReturningClause = S::HasReturningClause;
    type HasWhereClause = TypedTrue;
    type OutputFields = S::OutputFields;

    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        use std::fmt::Write;

        write!(f, " WHERE ")?;
        self.condition.write_sql_string(f, parameter_binder)
    }

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
        S: DeleteStatement<HasWhereClause = TypedFalse> + 'static,
        C: SqlExpression<S::DeleteFrom, SqlType = SqlBool> + 'static,
    > SqlStatement for DeleteWithWhereClause<S, C>
{
    impl_sql_statement_for_delete_statement! {}
}

/// A trait which allows filtering a delete statement so that it only deletes
/// records matching some condition.
pub trait FilterDeleteStatement: DeleteStatement<HasWhereClause = TypedFalse> {
    /// Filters this delete statement, so that it only deletes records which
    /// match the given condition.
    fn filter<C: SqlExpression<Self::DeleteFrom, SqlType = SqlBool>>(
        self,
        condition: C,
    ) -> DeleteWithWhereClause<Self, C> {
        DeleteWithWhereClause {
            statement: self,
            condition,
        }
    }
}

impl<T: DeleteStatement<HasWhereClause = TypedFalse>> FilterDeleteStatement for T {}

/// A wrapper around an sql delete statement which adds a `RETURNING` clause to
/// it.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`DeleteStatementReturning::returning`] function.
pub struct DeleteWithReturningClause<
    S: DeleteStatement<HasReturningClause = TypedFalse>,
    R: SelectedValues<S::DeleteFrom>,
> {
    statement: S,
    returning: R,
}

impl<S: DeleteStatement<HasReturningClause = TypedFalse>, R: SelectedValues<S::DeleteFrom>>
    DeleteStatement for DeleteWithReturningClause<S, R>
{
    type DeleteFrom = S::DeleteFrom;
    type HasReturningClause = TypedTrue;
    type HasWhereClause = S::HasWhereClause;
    type OutputFields = R::Fields;

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

    fn write_returning_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        use std::fmt::Write;

        write!(f, " RETURNING ")?;
        self.returning.write_sql_string(f, parameter_binder)
    }
}

impl<
        S: DeleteStatement<HasReturningClause = TypedFalse> + 'static,
        R: SelectedValues<S::DeleteFrom> + 'static,
    > SqlStatement for DeleteWithReturningClause<S, R>
{
    impl_sql_statement_for_delete_statement! {}
}

/// A trait which allows returning some values from the records deleted by some
/// delete statement.
pub trait DeleteStatementReturning: DeleteStatement<HasReturningClause = TypedFalse> {
    /// Selects some values to be returned from the records deleted by this
    /// delete statement. To provide a list of values to be returned, use the
    /// [`returning!`] macro.
    ///
    /// [`returning!`]: crate::returning
    fn returning<R: SelectedValues<Self::DeleteFrom>>(
        self,
        returning: R,
    ) -> DeleteWithReturningClause<Self, R> {
        DeleteWithReturningClause {
            statement: self,
            returning,
        }
    }
}

impl<T: DeleteStatement<HasReturningClause = TypedFalse>> DeleteStatementReturning for T {}
