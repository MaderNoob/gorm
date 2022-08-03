use super::SqlStatement;
use crate::{
    sql::{Insertable, ParameterBinder},
    Table, TypedConsListNil,
};

/// An sql insert statement
pub struct InsertStatement<I: Insertable>(I);
impl<I: Insertable> InsertStatement<I> {
    pub fn new(insertable: I) -> Self {
        Self(insertable)
    }
}

impl<I: Insertable + 'static> SqlStatement for InsertStatement<I> {
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
