use crate::error::*;
use async_trait::async_trait;
use sqlx::{Connection, Database, Executor};

use super::SqlStatementExecutor;

/// An asynchronous pool of database connections.
pub struct DatabaseConnection<DB: Database> {
    connection: DB::Connection,
}

impl<DB: Database> DatabaseConnection<DB> {
    /// Establish a new database connection.
    pub async fn connect(url: &str) -> Result<Self> {
        let connection = DB::Connection::connect(url).await?;
        Ok(Self { connection })
    }

    /// Establish a new database connection with the provided options.
    pub async fn connect_with(options: &<DB::Connection as Connection>::Options) -> Result<Self> {
        let connection = DB::Connection::connect_with(&options).await?;
        Ok(Self { connection })
    }
}

#[async_trait(?Send)]
impl<'a,DB: Database> SqlStatementExecutor for &'a mut DatabaseConnection<DB>
where
    for<'c> &'c mut DB::Connection: Executor<'c>,
{
    async fn execute(self, statement: impl crate::SqlStatement) -> Result<()> {
        let query_string = format!("{}", statement.formatter());
        self.connection.execute(query_string.as_str()).await?;
        Ok(())
    }
}
