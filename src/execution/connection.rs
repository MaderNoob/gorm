use deadpool_postgres::tokio_postgres::{Client, NoTls};

use super::transaction::DatabaseTransaction;
use crate::{
    error::*,
    execution::{impl_sql_statement_executor, ExecuteResult},
    sql::FromQueryResult,
    statements::SqlStatement,
};

/// A database connection.
pub struct DatabaseConnection {
    client: Client,
}

impl DatabaseConnection {
    /// Establish a new database connection to the given postgres connection
    /// url.
    pub async fn connect(url: &str) -> Result<Self> {
        let (client, connection) = deadpool_postgres::tokio_postgres::connect(url, NoTls).await?;

        // the connection must be awaited, run it in the background
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("database connection error: {}", e);
            }
        });

        Ok(Self { client })
    }

    /// Begins a transaction on this database connection.
    ///
    /// The transaction will roll back when dropped by default, use the
    /// [`DatabaseTransaction::commit`] function to commit it.
    pub async fn begin_transaction(&mut self) -> Result<DatabaseTransaction> {
        Ok(DatabaseTransaction {
            transaction: self.client.transaction().await?,
        })
    }
}

async fn get_raw_executor(database_connection: &DatabaseConnection) -> Result<&Client> {
    Ok(&database_connection.client)
}

impl_sql_statement_executor! {DatabaseConnection, get_raw_executor}
