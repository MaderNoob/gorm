use super::SqlStatement;
use crate::{
    sql::{Insertable, ParameterBinder, SelectedValues},
    Table, TypedConsListNil,
};

/// An sql insert statement.
///
/// This statement shouldn't be used directly, you should instead use the
/// [`Insertable::insert`] function.
pub struct InsertStatement<I: Insertable>(I);
impl<I: Insertable> InsertStatement<I> {
    /// Creates a new sql insert statement which inserts the given insertable.
    pub fn new(insertable: I) -> Self {
        Self(insertable)
    }

    /// Selects some values to be returned from this insert statement.
    pub fn returning<R: SelectedValues<I::Table>>(
        self,
        returning: R,
    ) -> ReturningInsertStatement<I, R> {
        ReturningInsertStatement::new(self.0, returning)
    }
}

impl<I: Insertable> SqlStatement for InsertStatement<I> {
    type OutputFields = TypedConsListNil;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        use std::fmt::Write;

        write!(f, "INSERT INTO {}(", <I::Table as Table>::TABLE_NAME,)?;
        self.0.write_value_names(f)?;
        write!(f, ") VALUES(")?;
        self.0.write_values(f, parameter_binder)?;
        write!(f, ")")?;

        Ok(())
    }
}

/// An sql insert statement with a returning clause.
///
/// This statement shouldn't be used directly, you should instead use the
/// [`Insertable::insert_returning`] function.
pub struct ReturningInsertStatement<I: Insertable, R: SelectedValues<I::Table>> {
    insertable: I,
    returning: R,
}

impl<I: Insertable, R: SelectedValues<I::Table>> ReturningInsertStatement<I, R> {
    /// Creates a new sql insert statement which inserts the given insertable and returns the given
    /// selected values.
    pub fn new(insertable: I, returning: R) -> Self {
        Self {
            insertable,
            returning,
        }
    }
}

impl<I: Insertable, R: SelectedValues<I::Table>> SqlStatement for ReturningInsertStatement<I, R> {
    type OutputFields = R::Fields;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        use std::fmt::Write;

        write!(f, "INSERT INTO {}(", <I::Table as Table>::TABLE_NAME,)?;
        self.insertable.write_value_names(f)?;
        write!(f, ") VALUES(")?;
        self.insertable.write_values(f, parameter_binder)?;
        write!(f, ") RETURNING ")?;
        self.returning.write_sql_string(f, parameter_binder)?;

        Ok(())
    }
}
