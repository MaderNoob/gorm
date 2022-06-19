use std::marker::PhantomData;

use crate::{expr::SqlExpression, selectable_tables::SelectableTables};

/// A condition in a where clause.
pub trait SqlCondition<S: SelectableTables> {
    /// Writes the condition as an sql string which can be used in a where clause.
    fn write_sql_string(self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

pub struct SqlConditionEq<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S>>(
    PhantomData<S>,
    PhantomData<Lhs>,
    PhantomData<Rhs>,
);
impl<S: SelectableTables, Lhs: SqlExpression<S>, Rhs: SqlExpression<S>>
    SqlConditionEq<S, Lhs, Rhs>
{
    pub fn new() -> Self {
        Self(PhantomData, PhantomData, PhantomData)
    }
}
