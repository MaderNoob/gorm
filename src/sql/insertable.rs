use super::ParameterBinder;
use crate::{
    error::*,
    execution::{ExecuteResult, SqlStatementExecutor},
    statements::{ExecuteSqlStatment, InsertStatement},
    Table,
};

#[async_trait::async_trait]
pub trait Insertable: Sized + 'static {
    type Table: Table;

    /// Writes the names of the values inserted by this insertable.
    /// For example, in the query:
    /// `INSERT INTO person(name,age) values('James', 29)`
    /// This represents the `name,age` part.
    fn write_value_names(&self, f: &mut String) -> std::fmt::Result;

    /// Writes the the values inserted by this insertable.
    /// For example, in the query:
    /// `INSERT INTO person(name,age) values('James', 29)`
    /// This represents the `'James',29` part.
    fn write_values<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    async fn insert(self, to: &(impl SqlStatementExecutor + Send + Sync)) -> Result<ExecuteResult> {
        InsertStatement::new(self).execute(to).await
    }
}
