mod connection;
mod transaction;

pub use connection::*;
use deadpool_postgres::{
    tokio_postgres::NoTls,
    Manager, ManagerConfig, Object, Pool,
};
pub use transaction::*;

use crate::{
    error::*,
    execution::{impl_sql_statement_executor, ExecuteResult},
    sql::FromQueryResult,
    statements::SqlStatement,
};

/// A database connection pool.
///
/// You can execute statements direclty on the pool, or get a connection from the pool using the
/// [`DatabaseConnectionPool::get`] function.
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

async fn get_raw_executor(database_connection_pool: &DatabaseConnectionPool) -> Result<Object> {
    Ok(database_connection_pool.pool.get().await?)
}

impl_sql_statement_executor! {DatabaseConnectionPool, get_raw_executor}
