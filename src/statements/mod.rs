mod create_table;
mod drop_table;
mod select;

use async_trait::async_trait;
pub use create_table::*;
pub use drop_table::*;
pub use select::*;

use crate::{
    error::*,
    execution::{ExecuteResult, SqlStatementExecutor},
    sql::{FieldsConsListItem, FromQueryResult, ParameterBinder},
};

/// An sql statement which can be executed by a database.
pub trait SqlStatement: Sized + 'static {
    /// The fields of the output of this statement.
    type OutputFields: FieldsConsListItem;

    /// Writes the sql statement as an sql string which can be executed by a
    /// database.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    fn build(&self) -> (String, ParameterBinder) {
        let mut parameter_binder = ParameterBinder::new();
        let mut query_string = String::new();
        self.write_sql_string(&mut query_string, &mut parameter_binder)
            .unwrap();
        println!("{:?}", (&query_string, parameter_binder.parameters()));
        (query_string, parameter_binder)
    }
}

#[async_trait]
pub trait ExecuteSqlStatment: SqlStatement {
    /// Executes the query on the given executor
    async fn execute(
        self,
        on: &(impl SqlStatementExecutor + Send + Sync),
    ) -> Result<ExecuteResult> {
        on.execute(self).await
    }

    async fn load_one<O: FromQueryResult<Fields = Self::OutputFields> + Send>(
        self,
        on: &(impl SqlStatementExecutor + Send + Sync),
    ) -> Result<O> {
        on.load_one(self).await
    }

    async fn load_optional<O: FromQueryResult<Fields = Self::OutputFields> + Send>(
        self,
        on: &(impl SqlStatementExecutor + Send + Sync),
    ) -> Result<Option<O>> {
        on.load_optional(self).await
    }

    async fn load_all<O: FromQueryResult<Fields = Self::OutputFields> + Send>(
        self,
        on: &(impl SqlStatementExecutor + Send + Sync),
    ) -> Result<Vec<O>> {
        on.load_all(self).await
    }
}

#[async_trait]
impl<S: SqlStatement> ExecuteSqlStatment for S {}
