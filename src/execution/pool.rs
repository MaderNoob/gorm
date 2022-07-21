use std::future::Future;

use crate::{
    error::*, fields_list::TypedConsListNil, from_query_result::FromQueryResult,
    util::TypesNotEqual, SqlStatement, 
};
use async_trait::async_trait;

use super::SqlStatementExecutor;

/// An asynchronous pool of database connections.
pub struct DatabaseConnectionPool<DB: Database> {
    pool: Pool<DB>,
}

impl<DB: Database> DatabaseConnectionPool<DB> {
    /// Creates a new connection pool with a default pool configuration and
    /// the given connection URI.
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = Pool::connect(url).await?;
        Ok(Self { pool })
    }

    /// Creates a new connection pool with a default pool configuration and
    /// the given connection options.
    pub async fn connect_with(
        options: <DB::Connection as sqlx::Connection>::Options,
    ) -> Result<Self> {
        let pool = Pool::connect_with(options).await?;
        Ok(Self { pool })
    }
}

#[async_trait(?Send)]
impl<'a, DB: Database + DatabaseBoundParametersFormatter<'a>> SqlStatementExecutor<'a>
    for &'a DatabaseConnectionPool<DB>
where
    &'a Pool<DB>: Executor<'a>,
{
    type SqlxExecutor = &'a Pool<DB>;
    type Database = DB;

    fn get_sqlx_executor(self) -> Self::SqlxExecutor {
        &self.pool
    }
}
