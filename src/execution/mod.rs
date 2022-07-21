use crate::sql::bound_parameters::ParameterBinder;
use futures::stream::StreamExt;

use async_trait::async_trait;

use crate::{
    error::*, fields_list::TypedConsListNil, from_query_result::FromQueryResult,
    util::TypesNotEqual, SqlStatement,
};

pub mod connection;
// pub mod pool;

/// An executor which can execute sql statements
#[async_trait]
pub trait SqlStatementExecutor: Sized {
    async fn execute(&self, statement: impl crate::SqlStatement + Send) -> Result<ExecuteResult>;

    async fn load_one<O: FromQueryResult, S: SqlStatement + Send>(&self, statement: S)
        -> Result<O>;

    async fn load_optional<O: FromQueryResult, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Option<O>>;

    // async fn load_all<S, O>(self, statement: S) -> Result<Vec<O>>
    // where
    //     S: SqlStatement,
    //     (S::OutputFields, TypedConsListNil): TypesNotEqual,
    //     O: for<'q> FromQueryResult<
    //         'q,
    //         <<Self::SqlxExecutor as Executor<'c>>::Database as Database>::Row,
    //         Fields = S::OutputFields,
    //     >,
    // {
    //     let mut bound_parameters_formatter = <Self::Database as DatabaseBoundParametersFormatter>::BoundParametersFormatter::new();
    //     let mut query_string = String::new();
    //     statement.write_sql_string(&mut query_string, &mut bound_parameters_formatter).unwrap();
    //     let sqlx_executor = self.get_sqlx_executor();

    //     let mut result = Vec::new();
    //     let mut rows_stream = sqlx_executor.fetch(query_string.as_str());
    //     while let Some(row_result) = rows_stream.next().await {
    //         let row = row_result?;
    //         result.push(O::from_row(&row)?);
    //     }
    //     Ok(result)
    // }
}

/// The result of executing an sql statement.
pub struct ExecuteResult {
    /// The amount of rows modified by the statement.
    pub rows_modified: u64,
}
