use async_trait::async_trait;
use tokio_postgres::types::{FromSql, FromSqlOwned};

use crate::{error::*, sql::FromQueryResult, statements::SqlStatement};

pub mod connection;

pub use connection::*;

/// An executor which can execute sql statements
#[async_trait]
pub trait SqlStatementExecutor: Sized {
    async fn execute(&self, statement: impl SqlStatement + Send) -> Result<ExecuteResult>;

    async fn load_one<O: FromQueryResult + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<O>;

    async fn load_one_one_column<O: FromSqlOwned + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<O>;

    async fn load_optional<O: FromQueryResult + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Option<O>>;

    async fn load_optional_one_column<O: FromSqlOwned + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Option<O>>;

    async fn load_all<O: FromQueryResult + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Vec<O>>;

    async fn load_all_one_column<O: FromSqlOwned + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Vec<O>>;
}

/// The result of executing an sql statement.
pub struct ExecuteResult {
    /// The amount of rows modified by the statement.
    pub rows_modified: u64,
}
