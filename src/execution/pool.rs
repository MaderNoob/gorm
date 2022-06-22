use crate::error::*;
use async_trait::async_trait;
use sqlx::{pool::PoolConnection, Executor, Database};

use super::SqlStatementExecutor;

/// An asynchronous pool of database connections.
pub struct DatabaseConnectionPool<DB: Database> {
    pool: sqlx::Pool<DB>,
}

impl<DB: Database> DatabaseConnectionPool<DB> {
    /// Creates a new connection pool with a default pool configuration and
    /// the given connection URI.
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = sqlx::Pool::connect(url).await?;
        Ok(Self { pool })
    }

    /// Creates a new connection pool with a default pool configuration and
    /// the given connection options.
    pub async fn connect_with(
        options: <DB::Connection as sqlx::Connection>::Options,
    ) -> Result<Self> {
        let pool = sqlx::Pool::connect_with(options).await?;
        Ok(Self { pool })
    }
}

#[async_trait(?Send)]
impl<'a, DB: Database> SqlStatementExecutor for &'a DatabaseConnectionPool<DB>
where
    for<'c> &'c mut PoolConnection<DB>: Executor<'c>,
{
    async fn execute(self, statement: impl crate::SqlStatement) -> Result<()> {
        let mut connection = self.pool.acquire().await?;
        let query_string = format!("{}", statement.formatter());
        connection.execute(query_string.as_str()).await?;
        Ok(())
    }
}
