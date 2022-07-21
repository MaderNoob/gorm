use std::future::Future;

use crate::{
    bound_parameters::ParameterBinder, error::*, from_query_result::FromQueryResult, ExecuteResult,
    SqlStatement,
};
use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};

use super::SqlStatementExecutor;

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
    async fn execute(&self, statement: impl crate::SqlStatement + Send) -> Result<ExecuteResult> {
        let mut parameter_binder = ParameterBinder::new();
        let mut query_string = String::new();
        statement
            .write_sql_string(&mut query_string, &mut parameter_binder)
            .unwrap();
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
        let mut parameter_binder = ParameterBinder::new();
        let mut query_string = String::new();
        statement
            .write_sql_string(&mut query_string, &mut parameter_binder)
            .unwrap();
        let row = self
            .client
            .query_one(&query_string, parameter_binder.parameters())
            .await?;

        Ok(O::from_row(row)?)
    }

    async fn load_optional<O: FromQueryResult, S: SqlStatement + Send>(
        &self,
        statement: S,
    ) -> Result<Option<O>> {
        let mut parameter_binder = ParameterBinder::new();
        let mut query_string = String::new();
        statement
            .write_sql_string(&mut query_string, &mut parameter_binder)
            .unwrap();
        let maybe_row = self
            .client
            .query_opt(&query_string, parameter_binder.parameters())
            .await?;

        match maybe_row {
            Some(row) => Ok(Some(O::from_row(row)?)),
            None => Ok(None),
        }
    }
}
