//! Execution of sql statements.
//!
//! This module provides different ways to execute sql statements - connections, transactions,
//! connection pools.
//!
//! The [`SqlStatementExecutor`] trait is the core of this module, and this module provides
//! different types that implement it.

use async_trait::async_trait;

use crate::{
    error::*,
    sql::FromQueryResult,
    statements::SqlStatement,
    TypedConsListNil, TypesNotEqual,
};

mod connection;
mod connection_pool;
mod transaction;

pub use connection::*;
pub use connection_pool::*;
pub use transaction::*;

/// An executor which can execute sql statements.
#[async_trait]
pub trait SqlStatementExecutor: Sized + Send + Sync {
    /// Executes the given sql statement.
    async fn execute(&self, statement: impl SqlStatement + Send) -> Result<ExecuteResult>;

    /// Executes the given sql statement and loads the first returned row from
    /// it.
    async fn load_one<O: FromQueryResult + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<O>
    where
        (S::OutputFields, TypedConsListNil): TypesNotEqual;

    /// Executes the given sql statement and loads the first column of the first
    /// returned row from it.
    async fn load_one_value<
        FieldName: crate::sql::FieldNameCharsConsListItem,
        FieldType: deadpool_postgres::tokio_postgres::types::FromSqlOwned + Send,
        S: SqlStatement<
                OutputFields = crate::sql::FieldsConsListCons<
                    FieldName,
                    FieldType,
                    crate::util::TypedConsListNil,
                >,
            > + Send,
    >(
        &self,
        statement: S,
    ) -> Result<FieldType>;

    // pub trait LoadSingleColumnSqlStatment<
    //     FieldName: FieldNameCharsConsListItem,
    //     FieldType: FromSqlOwned + Send,
    // >: SqlStatement<OutputFields = FieldsConsListCons<FieldName, FieldType,
    // TypedConsListNil>>

    /// Executes the given sql statement and loads the first returned row from
    /// it, if any.
    async fn load_optional<O: FromQueryResult + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Option<O>>
    where
        (S::OutputFields, TypedConsListNil): TypesNotEqual;

    /// Executes the given sql statement and loads the first column first
    /// returned row from it, if any.
    async fn load_optional_value<
        FieldName: crate::sql::FieldNameCharsConsListItem,
        FieldType: deadpool_postgres::tokio_postgres::types::FromSqlOwned + Send,
        S: SqlStatement<
                OutputFields = crate::sql::FieldsConsListCons<
                    FieldName,
                    FieldType,
                    crate::util::TypedConsListNil,
                >,
            > + Send,
    >(
        &self,
        statement: S,
    ) -> Result<Option<FieldType>>;

    /// Executes the given sql statement and loads all the rows returned from
    /// it.
    async fn load_all<O: FromQueryResult + Send, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Vec<O>>
    where
        (S::OutputFields, TypedConsListNil): TypesNotEqual;

    /// Executes the given sql statement and loads the first column of all the
    /// rows returned from it.
    async fn load_all_values<
        FieldName: crate::sql::FieldNameCharsConsListItem,
        FieldType: deadpool_postgres::tokio_postgres::types::FromSqlOwned + Send,
        S: SqlStatement<
                OutputFields = crate::sql::FieldsConsListCons<
                    FieldName,
                    FieldType,
                    crate::util::TypedConsListNil,
                >,
            > + Send,
    >(
        &self,
        statement: S,
    ) -> Result<Vec<FieldType>>;
}

/// The result of executing an sql statement.
pub struct ExecuteResult {
    /// The amount of rows modified by the statement.
    pub rows_modified: u64,
}

/// Implements the [`SqlStatementExecutor`] trait for some type, given a
/// function which returns its raw executor, and its generics.
macro_rules! impl_sql_statement_executor {
    {$impl_for: ty, $get_raw_executor: expr $(,$($generic:tt),+)?} => {
        #[async_trait::async_trait]
        impl $(< $($generic),+ >)? crate::execution::SqlStatementExecutor for $impl_for {
            async fn execute(
                &self,
                statement: impl crate::statements::SqlStatement + Send,
            ) -> Result<ExecuteResult> {
                let (query_string, parameter_binder) = statement.build();
                let rows_modified = $get_raw_executor(self).await?
                    .execute(&query_string, parameter_binder.parameters())
                    .await?;

                Ok(ExecuteResult { rows_modified })
            }

            async fn load_one<O: FromQueryResult, S: SqlStatement + Send>(
                &self,
                statement: S,
            ) -> Result<O>
            where
                (S::OutputFields, crate::util::TypedConsListNil): crate::util::TypesNotEqual
            {
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_raw_executor(self).await?
                    .query_raw(&query_string, parameter_binder.parameters().iter().copied())
                    .await?;

                pin_mut!(row_stream);

                let maybe_row = row_stream.try_next().await?;
                match maybe_row {
                    Some(row) => Ok(O::from_row(row)?),
                    None => Err(Error::NoRecords),
                }
            }

            async fn load_one_value<
                FieldName: crate::sql::FieldNameCharsConsListItem,
                FieldType: deadpool_postgres::tokio_postgres::types::FromSqlOwned + Send,
                S: SqlStatement<
                        OutputFields = crate::sql::FieldsConsListCons<
                            FieldName,
                            FieldType,
                            crate::util::TypedConsListNil,
                        >,
                    > + Send,
            >(
                &self,
                statement: S,
            ) -> Result<FieldType> {
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_raw_executor(self).await?
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
            ) -> Result<Option<O>>
            where
                (S::OutputFields, crate::util::TypedConsListNil): crate::util::TypesNotEqual
            {
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_raw_executor(self).await?
                    .query_raw(&query_string, parameter_binder.parameters().iter().copied())
                    .await?;

                pin_mut!(row_stream);

                let maybe_row = row_stream.try_next().await?;
                match maybe_row {
                    Some(row) => Ok(Some(O::from_row(row)?)),
                    None => Ok(None),
                }
            }

            async fn load_optional_value<
                FieldName: crate::sql::FieldNameCharsConsListItem,
                FieldType: deadpool_postgres::tokio_postgres::types::FromSqlOwned + Send,
                S: SqlStatement<
                        OutputFields = crate::sql::FieldsConsListCons<
                            FieldName,
                            FieldType,
                            crate::util::TypedConsListNil,
                        >,
                    > + Send,
            >(
                &self,
                statement: S,
            ) -> Result<Option<FieldType>> {
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_raw_executor(self).await?
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
            ) -> Result<Vec<O>>
            where
                (S::OutputFields, crate::util::TypedConsListNil): crate::util::TypesNotEqual
            {
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_raw_executor(self).await?
                    .query_raw(&query_string, parameter_binder.parameters().iter().copied())
                    .await?;

                pin_mut!(row_stream);

                let mut records = Vec::new();
                while let Some(row) = row_stream.try_next().await? {
                    records.push(O::from_row(row)?)
                }
                Ok(records)
            }

            async fn load_all_values<
                FieldName: crate::sql::FieldNameCharsConsListItem,
                FieldType: deadpool_postgres::tokio_postgres::types::FromSqlOwned + Send,
                S: SqlStatement<
                        OutputFields = crate::sql::FieldsConsListCons<
                            FieldName,
                            FieldType,
                            crate::util::TypedConsListNil,
                        >,
                    > + Send,
            >(
                &self,
                statement: S,
            ) -> Result<Vec<FieldType>> {
                use futures::{pin_mut, TryStreamExt};

                let (query_string, parameter_binder) = statement.build();
                let row_stream = $get_raw_executor(self).await?
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
