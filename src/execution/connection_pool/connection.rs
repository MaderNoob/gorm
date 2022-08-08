use deadpool_postgres::Object;

use super::transaction::DatabaseTransactionFromPool;
use crate::{
    error::*,
    execution::{impl_sql_statement_executor, ExecuteResult},
    statements::SqlStatement,
    FromQueryResult,
};

/// A database connection from a connection pool.
pub struct DatabaseConnectionFromPool {
    pub(super) client: Object,
}

impl DatabaseConnectionFromPool {
    /// Begins a transaction on this database connection.
    ///
    /// The transaction will roll back when dropped by default, use the
    /// [`DatabaseTransactionFromPool::commit`] function to commit it.
    pub async fn begin_transaction(&mut self) -> Result<DatabaseTransactionFromPool> {
        Ok(DatabaseTransactionFromPool {
            transaction: self.client.transaction().await?,
        })
    }
}

async fn get_raw_executor(database_connection: &DatabaseConnectionFromPool) -> Result<&Object> {
    Ok(&database_connection.client)
}

impl_sql_statement_executor! {DatabaseConnectionFromPool, get_raw_executor}
