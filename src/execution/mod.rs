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

macro_rules! impl_sql_statement_executor {
    {$impl_for: ty, $get_client: expr $(,$($generic:tt),+)?} => {
        #[async_trait::async_trait]
        impl $(< $($generic),+ >)? crate::execution::SqlStatementExecutor for $impl_for {
            async fn execute(
                &self,
                statement: impl crate::statements::SqlStatement + Send,
            ) -> Result<ExecuteResult> {
                let (query_string, parameter_binder) = statement.build();
                let rows_modified = $get_client(self).await?
                    .execute(&query_string, parameter_binder.parameters())
                    .await?;

                Ok(ExecuteResult { rows_modified })
            }

            async fn load_one<O: FromQueryResult, S: SqlStatement + Send>(
                &self,
                statement: S,
            ) -> Result<O> {
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_client(self).await?
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
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_client(self).await?
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
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_client(self).await?
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
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_client(self).await?
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
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_client(self).await?
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
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_client(self).await?
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
    };
}

pub(in crate::execution) use impl_sql_statement_executor;
