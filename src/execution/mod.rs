use async_trait::async_trait;
use deadpool_postgres::tokio_postgres::types::FromSqlOwned;

use crate::{error::*, sql::FromQueryResult, statements::SqlStatement};

mod connection;
mod connection_pool;
mod transaction;

pub use connection::*;
pub use connection_pool::*;
pub use transaction::*;

/// An executor which can execute sql statements
#[async_trait]
pub trait SqlStatementExecutor: Sized + Send + Sync {
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
