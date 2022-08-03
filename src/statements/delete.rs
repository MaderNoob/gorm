use std::marker::PhantomData;

use super::SqlStatement;
use crate::{
    sql::{ParameterBinder, SqlCondition},
    Table, TypedConsListNil,
};

/// An sql delete statement
pub struct DeleteStatement<T: Table>(PhantomData<T>);
impl<T: Table> DeleteStatement<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// Only delete the items matching the given condition
    pub fn filter<C: SqlCondition<T>>(self, condition: C) -> FilteredDeleteStatement<T, C> {
        FilteredDeleteStatement::new(condition)
    }
}

impl<T: Table> SqlStatement for DeleteStatement<T> {
    type OutputFields = TypedConsListNil;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        use std::fmt::Write;

        write!(f, "DELETE FROM {}", T::TABLE_NAME)
    }
}

pub struct FilteredDeleteStatement<T: Table, C: SqlCondition<T>> {
    condition: C,
    phantom: PhantomData<T>,
}

impl<T: Table, C: SqlCondition<T>> FilteredDeleteStatement<T, C> {
    pub fn new(condition: C) -> Self {
        Self {
            condition,
            phantom: PhantomData,
        }
    }
}

impl<T: Table, C: SqlCondition<T> + 'static> SqlStatement for FilteredDeleteStatement<T, C> {
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

        write!(f, "DELETE FROM {} WHERE ", T::TABLE_NAME)?;
        self.condition.write_sql_string(f, parameter_binder)
    }
}
