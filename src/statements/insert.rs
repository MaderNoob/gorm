use super::SqlStatement;
use crate::{
    sql::{
        FieldsConsListItem, Insertable, ParameterBinder, SelectedValues, UniqueConstraint,
        UpdateSet,
    },
    util::{TypedBool, TypedConsListNil, TypedFalse, TypedTrue},
    Table,
};

/// Represents any type of sql insert statement.
pub trait InsertStatement: Sized {
    /// A type identifying the output fields of this insert statement, selected
    /// in its `RETURNING` clause.
    type OutputFields: FieldsConsListItem;

    /// Does this insert statement have a `RETURNING` clause?
    type HasReturningClause: TypedBool;

    /// Does this insert statement have an `ON CONFLICT` clause?
    type HasOnConflictClause: TypedBool;

    /// The insertable which this insert statement inserts.
    type Insertable: Insertable;

    fn get_insertable(&self) -> &Self::Insertable;

    /// Writes the `RETURNING` clause of this insert statement.
    fn write_returning_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes the `ON CONFLICT` clause of this insert statement.
    fn write_on_conflict_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Writes this insert statement as an sql string.
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
            "INSERT INTO {}(",
            <<Self::Insertable as Insertable>::Table as Table>::TABLE_NAME,
        )?;
        self.get_insertable().write_value_names(f)?;
        write!(f, ") VALUES(")?;
        self.get_insertable().write_values(f, parameter_binder)?;
        write!(f, ")")?;
        self.write_on_conflict_clause(f, parameter_binder)?;
        self.write_returning_clause(f, parameter_binder)?;

        Ok(())
    }
}

/// Implements the [`SqlStatement`] trait for some type which implements
/// [`InsertStatement`]
macro_rules! impl_sql_statement_for_insert_statement {
    {} => {
        type OutputFields = <Self as InsertStatement>::OutputFields;

        fn write_sql_string<'s, 'a>(
            &'s self,
            f: &mut String,
            parameter_binder: &mut ParameterBinder<'a>,
        ) -> std::fmt::Result
        where
            's: 'a,
        {
            <Self as InsertStatement>::write_sql_string(&self, f, parameter_binder)
        }
    };
}

/// An empty sql insert statement.
///
/// This statement shouldn't be used directly, you should instead use the
/// [`Insertable::insert`] function.
pub struct EmptyInsertStatement<I: Insertable>(I);
impl<I: Insertable> EmptyInsertStatement<I> {
    /// Creates a new sql insert statement which inserts the given insertable.
    pub fn new(insertable: I) -> Self {
        Self(insertable)
    }
}

impl<I: Insertable> InsertStatement for EmptyInsertStatement<I> {
    type HasOnConflictClause = TypedFalse;
    type HasReturningClause = TypedFalse;
    type Insertable = I;
    type OutputFields = TypedConsListNil;

    fn get_insertable(&self) -> &Self::Insertable {
        &self.0
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

    fn write_on_conflict_clause<'s, 'a>(
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

impl<I: Insertable> SqlStatement for EmptyInsertStatement<I> {
    impl_sql_statement_for_insert_statement! {}
}

/// A wrapper around an sql insert statement which adds a `RETURNING` clause to
/// it.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`InsertStatementReturning::returning`] function.
pub struct InsertWithReturningClause<
    S: InsertStatement<HasReturningClause = TypedFalse>,
    R: SelectedValues<<S::Insertable as Insertable>::Table>,
> {
    statement: S,
    returning: R,
}

impl<
    S: InsertStatement<HasReturningClause = TypedFalse>,
    R: SelectedValues<<S::Insertable as Insertable>::Table>,
> InsertStatement for InsertWithReturningClause<S, R>
{
    type HasOnConflictClause = S::HasOnConflictClause;
    type HasReturningClause = TypedTrue;
    type Insertable = S::Insertable;
    type OutputFields = R::Fields;

    fn get_insertable(&self) -> &Self::Insertable {
        self.statement.get_insertable()
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

    fn write_on_conflict_clause<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        self.statement.write_on_conflict_clause(f, parameter_binder)
    }
}

impl<
    S: InsertStatement<HasReturningClause = TypedFalse>,
    R: SelectedValues<<S::Insertable as Insertable>::Table>,
> SqlStatement for InsertWithReturningClause<S, R>
{
    impl_sql_statement_for_insert_statement! {}
}

/// A trait which allows returning some values from the records inserted by some
/// insert statement.
pub trait InsertStatementReturning: InsertStatement<HasReturningClause = TypedFalse> {
    /// Selects some values to be returned from the records inserted by this
    /// insert statement. To provide a list of values to be returned, use the
    /// [`returning!`] macro.
    ///
    /// [`returning!`]: crate::returning
    fn returning<R: SelectedValues<<Self::Insertable as Insertable>::Table>>(
        self,
        returning: R,
    ) -> InsertWithReturningClause<Self, R> {
        InsertWithReturningClause {
            statement: self,
            returning,
        }
    }
}

impl<T: InsertStatement<HasReturningClause = TypedFalse>> InsertStatementReturning for T {}

/// A wrapper around an sql insert statement which allows adding an `ON
/// CONFLICT` clause to it, by specifying the updates that should be performed
/// on a conflicting row in case of conflict, using the
/// [`InsertWithOnConflictClauseBuilder::do_update`] function.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`InsertStatementOnConflict::on_conflict`] function.
pub struct InsertWithOnConflictClauseBuilder<
    S: InsertStatement<HasOnConflictClause = TypedFalse>,
    C: UniqueConstraint<Table = <S::Insertable as Insertable>::Table>,
> {
    statement: S,
    constraint: C,
}

impl<
    S: InsertStatement<HasOnConflictClause = TypedFalse>,
    C: UniqueConstraint<Table = <S::Insertable as Insertable>::Table>,
> InsertWithOnConflictClauseBuilder<S, C>
{
    /// Adds an update set to be applied to a conflicting row in case
    /// of a conflict when inserting this value. To provide an update set to be
    /// applied in case of a conflict, use the [`update_set!`] macro.
    ///
    /// [`update_set!`]: gorm_macros::update_set
    pub fn do_update<U: UpdateSet<UpdateTable = <S::Insertable as Insertable>::Table>>(
        self,
        update_set: U,
    ) -> InsertWithOnConflictClause<S, C, U> {
        InsertWithOnConflictClause {
            statement: self.statement,
            _constraint: self.constraint,
            update_set,
        }
    }
}

/// A trait which allows updating an existing row when a conflict occurs while
/// trying to insert values into some table.
pub trait InsertStatementOnConflict: InsertStatement<HasOnConflictClause = TypedFalse> {
    /// Returns a builder for an on conflict clause, using a unique constraint
    /// to detect the conflict.
    ///
    /// The returned builder allows specifying what updates should be performed
    /// on the conflicting row in case of a conflict, by using the
    /// [`InsertWithOnConflictClauseBuilder::do_update`] function.
    fn on_conflict<C: UniqueConstraint<Table = <Self::Insertable as Insertable>::Table>>(
        self,
        constraint: C,
    ) -> InsertWithOnConflictClauseBuilder<Self, C> {
        InsertWithOnConflictClauseBuilder {
            statement: self,
            constraint,
        }
    }
}

impl<T: InsertStatement<HasOnConflictClause = TypedFalse>> InsertStatementOnConflict for T {}

/// A wrapper around an sql insert statement which adds an `ON CONFLICT` clause
/// to it.
///
/// This wrapper shouldn't be used directly, you should instead use the
/// [`InsertStatementOnConflict::on_conflict`], and then the
/// [`InsertWithOnConflictClauseBuilder::do_update`] functions.
pub struct InsertWithOnConflictClause<
    S: InsertStatement<HasOnConflictClause = TypedFalse>,
    C: UniqueConstraint<Table = <S::Insertable as Insertable>::Table>,
    U: UpdateSet<UpdateTable = <S::Insertable as Insertable>::Table>,
> {
    statement: S,
    _constraint: C,
    update_set: U,
}

impl<
    S: InsertStatement<HasOnConflictClause = TypedFalse>,
    C: UniqueConstraint<Table = <S::Insertable as Insertable>::Table>,
    U: UpdateSet<UpdateTable = <S::Insertable as Insertable>::Table>,
> InsertStatement for InsertWithOnConflictClause<S, C, U>
{
    type HasOnConflictClause = TypedTrue;
    type HasReturningClause = S::HasReturningClause;
    type Insertable = S::Insertable;
    type OutputFields = S::OutputFields;

    fn get_insertable(&self) -> &Self::Insertable {
        self.statement.get_insertable()
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

    fn write_on_conflict_clause<'s, 'a>(
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
            " ON CONFLICT({}) DO UPDATE SET ",
            C::FIELDS_COMMA_SEPERATED
        )?;
        self.update_set.write_sql_string(f, parameter_binder)
    }
}

impl<
    S: InsertStatement<HasOnConflictClause = TypedFalse>,
    C: UniqueConstraint<Table = <S::Insertable as Insertable>::Table>,
    U: UpdateSet<UpdateTable = <S::Insertable as Insertable>::Table>,
> SqlStatement for InsertWithOnConflictClause<S, C, U>
{
    impl_sql_statement_for_insert_statement! {}
}
