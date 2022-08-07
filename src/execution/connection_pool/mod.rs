mod connection;
mod transaction;

use async_trait::async_trait;
pub use connection::*;
use deadpool_postgres::{
    tokio_postgres::{types::FromSqlOwned, NoTls},
    Manager, ManagerConfig, Pool,
};
use futures::{pin_mut, TryStreamExt};
pub use transaction::*;

use super::SqlStatementExecutor;
use crate::{error::*, execution::ExecuteResult, sql::FromQueryResult, statements::SqlStatement};

/// An database connection.
pub struct DatabaseConnectionPool {
    pool: Pool,
}

impl DatabaseConnectionPool {
    /// Create a new database connection pool with the given postgres connection
    /// url.
    pub async fn connect(url: &str) -> Result<Self> {
        let tokio_postgres_config: deadpool_postgres::tokio_postgres::Config = url.parse()?;
        let manager = Manager::from_config(
            tokio_postgres_config,
            NoTls,
            ManagerConfig {
                recycling_method: deadpool_postgres::RecyclingMethod::Fast,
            },
        );
        let pool = Pool::builder(manager).build()?;

        Ok(Self { pool })
    }

    /// Returns a single connection from the connection pool.
    pub async fn get(&self) -> Result<DatabaseConnectionFromPool> {
        let client = self.pool.get().await?;
        Ok(DatabaseConnectionFromPool { client })
    }
}

#[async_trait]
impl SqlStatementExecutor for DatabaseConnectionPool {
    async fn execute(
        &self,
        statement: impl crate::statements::SqlStatement + Send,
    ) -> Result<ExecuteResult> {
        let client = self.pool.get().await?;
        let (query_string, parameter_binder) = statement.build();
        let rows_modified = client
            .execute(&query_string, parameter_binder.parameters())
            .await?;

        Ok(ExecuteResult { rows_modified })
    }

    async fn load_one<O: FromQueryResult, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<O> {
        let client = self.pool.get().await?;
        let (query_string, parameter_binder) = statement.build();
        let row_stream = client
            .query_raw(&query_string, parameter_binder.parameters().iter().copied())
            .await?;

        pin_mut!(row_stream);

        let maybe_row = row_stream.try_next().await?;
        match maybe_row {
            Some(row) => Ok(O::from_row(row)?),
            None => Err(Error::NoRecords),
        }
    }

    async fn load_one_one_column<O: FromSqlOwned + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<O> {
        let client = self.pool.get().await?;
        let (query_string, parameter_binder) = statement.build();
        let row_stream = client
            .query_raw(&query_string, parameter_binder.parameters().iter().copied())
            .await?;

        pin_mut!(row_stream);

        let maybe_row = row_stream.try_next().await?;
        match maybe_row {
            Some(row) => Ok(row.try_get(0)?),
            None => Err(Error::NoRecords),
        }
    }

    async fn load_optional<O: FromQueryResult, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Option<O>> {
        let client = self.pool.get().await?;
        let (query_string, parameter_binder) = statement.build();
        let row_stream = client
            .query_raw(&query_string, parameter_binder.parameters().iter().copied())
            .await?;

        pin_mut!(row_stream);

        let maybe_row = row_stream.try_next().await?;
        match maybe_row {
            Some(row) => Ok(Some(O::from_row(row)?)),
            None => Ok(None),
        }
    }

    async fn load_optional_one_column<O: FromSqlOwned + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Option<O>> {
        let client = self.pool.get().await?;
        let (query_string, parameter_binder) = statement.build();
        let row_stream = client
            .query_raw(&query_string, parameter_binder.parameters().iter().copied())
            .await?;

        pin_mut!(row_stream);

        let maybe_row = row_stream.try_next().await?;
        match maybe_row {
            Some(row) => Ok(Some(row.try_get(0)?)),
            None => Ok(None),
        }
    }

    async fn load_all<O: FromQueryResult + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Vec<O>> {
        let client = self.pool.get().await?;
        let (query_string, parameter_binder) = statement.build();
        let row_stream = client
            .query_raw(&query_string, parameter_binder.parameters().iter().copied())
            .await?;

        pin_mut!(row_stream);

        let mut records = Vec::new();
        while let Some(row) = row_stream.try_next().await? {
            records.push(O::from_row(row)?)
        }
        Ok(records)
    }

    async fn load_all_one_column<O: FromSqlOwned + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Vec<O>> {
        let client = self.pool.get().await?;
        let (query_string, parameter_binder) = statement.build();
        let row_stream = client
            .query_raw(&query_string, parameter_binder.parameters().iter().copied())
            .await?;

        pin_mut!(row_stream);

        let mut records = Vec::new();
        while let Some(row) = row_stream.try_next().await? {
            records.push(row.try_get(0)?)
        }
        Ok(records)
    }
}
