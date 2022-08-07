use super::SqlStatement;
use crate::{
    sql::{Insertable, ParameterBinder, SelectedValues},
    Table, TypedConsListNil,
};

/// An sql insert statement
pub struct InsertStatement<I: Insertable>(I);
impl<I: Insertable> InsertStatement<I> {
    pub fn new(insertable: I) -> Self {
        Self(insertable)
    }

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

pub struct ReturningInsertStatement<I: Insertable, R: SelectedValues<I::Table>> {
    insertable: I,
    returning: R,
}

impl<I: Insertable, R: SelectedValues<I::Table>> ReturningInsertStatement<I, R> {
    pub fn new(insertable: I, returning: R) -> Self {
        Self {
            insertable,
            returning,
        }
    }
}

impl<I: Insertable, R: SelectedValues<I::Table>> SqlStatement
    for ReturningInsertStatement<I, R>
{
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
