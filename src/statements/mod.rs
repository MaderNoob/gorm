mod create_table;
mod delete;
mod drop_table;
mod insert;
mod select;

use async_trait::async_trait;
pub use create_table::*;
use deadpool_postgres::tokio_postgres::types::FromSqlOwned;
pub use delete::*;
pub use drop_table::*;
pub use insert::*;
pub use select::*;

use crate::{
    error::*,
    execution::{ExecuteResult, SqlStatementExecutor},
    sql::{
        FieldNameCharsConsListItem, FieldsConsListCons, FieldsConsListItem, FromQueryResult,
        ParameterBinder,
    },
    TypedConsListNil, TypesNotEqual,
};

/// An sql statement which can be executed by a database.
pub trait SqlStatement: Sized {
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
        (query_string, parameter_binder)
    }
}

#[async_trait]
pub trait ExecuteSqlStatment: SqlStatement {
    /// Executes the query on the given executor
    async fn execute(self, on: &impl SqlStatementExecutor) -> Result<ExecuteResult> {
        on.execute(self).await
    }
}

#[async_trait]
impl<S: SqlStatement> ExecuteSqlStatment for S {}

#[async_trait]
pub trait LoadSqlStatment: SqlStatement
where
    (Self::OutputFields, TypedConsListNil): TypesNotEqual,
{
    async fn load_one<O: FromQueryResult<Fields = Self::OutputFields> + Send>(
        self,
        on: &impl SqlStatementExecutor,
    ) -> Result<O> {
        on.load_one(self).await
    }

    async fn load_optional<O: FromQueryResult<Fields = Self::OutputFields> + Send>(
        self,
        on: &impl SqlStatementExecutor,
    ) -> Result<Option<O>> {
        on.load_optional(self).await
    }

    async fn load_all<O: FromQueryResult<Fields = Self::OutputFields> + Send>(
        self,
        on: &impl SqlStatementExecutor,
    ) -> Result<Vec<O>> {
        on.load_all(self).await
    }
}

#[async_trait]
impl<S: SqlStatement> LoadSqlStatment for S where (S::OutputFields, TypedConsListNil): TypesNotEqual {}

#[async_trait]
pub trait LoadSingleColumnSqlStatment<
    FieldName: FieldNameCharsConsListItem,
    FieldType: FromSqlOwned + Send,
>: SqlStatement<OutputFields = FieldsConsListCons<FieldName, FieldType, TypedConsListNil>>
{
    async fn load_one_value(self, on: &impl SqlStatementExecutor) -> Result<FieldType> {
        on.load_one_value(self).await
    }

    async fn load_optional_value(
        self,
        on: &impl SqlStatementExecutor,
    ) -> Result<Option<FieldType>> {
        on.load_optional_value(self).await
    }

    async fn load_all_values(self, on: &impl SqlStatementExecutor) -> Result<Vec<FieldType>> {
        on.load_all_values(self).await
    }
}

#[async_trait]
impl<
    FieldName: FieldNameCharsConsListItem,
    FieldType: FromSqlOwned + Send,
    S: SqlStatement<OutputFields = FieldsConsListCons<FieldName, FieldType, TypedConsListNil>>,
> LoadSingleColumnSqlStatment<FieldName, FieldType> for S
{
}
