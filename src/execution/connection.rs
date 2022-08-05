use async_trait::async_trait;
use futures::{pin_mut, TryStreamExt};
use tokio_postgres::{types::FromSqlOwned, Client, NoTls};

use super::SqlStatementExecutor;
use crate::{error::*, execution::ExecuteResult, sql::FromQueryResult, statements::SqlStatement};

/// An database connection.
pub struct DatabaseConnection {
    client: Client,
}

impl DatabaseConnection {
    /// Establish a new database connection.
    pub async fn connect(url: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(url, NoTls).await?;

        // the connection must be awaited, run it in the background
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("database connection error: {}", e);
            }
        });

        Ok(Self { client })
    }
}

#[async_trait]
impl SqlStatementExecutor for DatabaseConnection {
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
