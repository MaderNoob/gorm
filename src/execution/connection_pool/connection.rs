use async_trait::async_trait;
use deadpool_postgres::{tokio_postgres::types::FromSqlOwned, Object};
use futures::{pin_mut, TryStreamExt};

use super::transaction::DatabaseTransactionFromPool;
use crate::{
    error::*,
    execution::{ExecuteResult, SqlStatementExecutor},
    statements::SqlStatement,
    FromQueryResult,
};

/// An database connection.
pub struct DatabaseConnectionFromPool {
    pub(super) client: Object,
}

impl DatabaseConnectionFromPool {
    /// Begins a transaction on this database connection.
    ///
    /// The transaction will roll back when dropped by default, use the `commit`
    /// method to commit it.
    pub async fn begin_transaction(&mut self) -> Result<DatabaseTransactionFromPool> {
        Ok(DatabaseTransactionFromPool {
            transaction: self.client.transaction().await?,
        })
    }
}

#[async_trait]
impl SqlStatementExecutor for DatabaseConnectionFromPool {
    async fn execute(
        &self,
        statement: impl crate::statements::SqlStatement + Send,
    ) -> Result<ExecuteResult> {
        let (query_string, parameter_binder) = statement.build();
        let rows_modified = self
            .client
            .execute(&query_string, parameter_binder.parameters())
            .await?;

        Ok(ExecuteResult { rows_modified })
    }

    async fn load_one<O: FromQueryResult, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<O> {
        let (query_string, parameter_binder) = statement.build();
        let row_stream = self
            .client
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
        let (query_string, parameter_binder) = statement.build();
        let row_stream = self
            .client
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
        let (query_string, parameter_binder) = statement.build();
        let row_stream = self
            .client
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
        let (query_string, parameter_binder) = statement.build();
        let row_stream = self
            .client
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
        let (query_string, parameter_binder) = statement.build();
        let row_stream = self
            .client
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
        let (query_string, parameter_binder) = statement.build();
        let row_stream = self
            .client
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
