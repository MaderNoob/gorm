use std::marker::PhantomData;

use super::SqlStatement;
use crate::{
    sql::{FieldsConsListItem, ParameterBinder, SelectedValues, SqlCondition},
    Table, TypedBool, TypedConsListNil, TypedFalse, TypedTrue,
};

pub trait DeleteStatement: Sized {
    type OutputFields: FieldsConsListItem;
    type DeleteFrom: Table;

    type HasWhereClause: TypedBool;
    type HasReturningClause: TypedBool;

    fn write_where_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    fn write_returning_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

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

/// An sql delete statement
pub struct EmptyDeleteStatement<T: Table>(PhantomData<T>);
impl<T: Table> EmptyDeleteStatement<T> {
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

pub struct DeleteWithWhereClause<
    S: DeleteStatement<HasWhereClause = TypedFalse>,
    C: SqlCondition<S::DeleteFrom>,
> {
    statement: S,
    condition: C,
}

impl<S: DeleteStatement<HasWhereClause = TypedFalse>, C: SqlCondition<S::DeleteFrom>>
    DeleteStatement for DeleteWithWhereClause<S, C>
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
    C: SqlCondition<S::DeleteFrom> + 'static,
> SqlStatement for DeleteWithWhereClause<S, C>
{
    impl_sql_statement_for_delete_statement! {}
}

pub trait FilterDeleteStatement: DeleteStatement<HasWhereClause = TypedFalse> {
    fn filter<C: SqlCondition<Self::DeleteFrom>>(
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

pub struct WithReturningClause<
    S: DeleteStatement<HasReturningClause = TypedFalse>,
    R: SelectedValues<S::DeleteFrom>,
> {
    statement: S,
    returning: R,
}

impl<S: DeleteStatement<HasReturningClause = TypedFalse>, R: SelectedValues<S::DeleteFrom>>
    DeleteStatement for WithReturningClause<S, R>
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
> SqlStatement for WithReturningClause<S, R>
{
    impl_sql_statement_for_delete_statement! {}
}

pub trait Returning: DeleteStatement<HasReturningClause = TypedFalse> {
    fn returning<R: SelectedValues<Self::DeleteFrom>>(
        self,
        returning: R,
    ) -> WithReturningClause<Self, R> {
        WithReturningClause {
            statement: self,
            returning,
        }
    }
}

impl<T: DeleteStatement<HasReturningClause = TypedFalse>> Returning for T {}
