use thiserror::Error;

/// An error types which encapsulates all errors which can occur while using
/// this crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("database error")]
    DatabaseError(
        #[from]
        #[source]
        deadpool_postgres::tokio_postgres::Error,
    ),

    #[error("database connection pool build error")]
    DatabaseConnectionPoolBuildError(
        #[from]
        #[source]
        deadpool_postgres::BuildError,
    ),

    #[error("database connection pool error")]
    DatabaseConnectionPoolError(
        #[from]
        #[source]
        deadpool_postgres::PoolError,
    ),

    #[error("failed to get column from query results: {0}")]
    FailedToGetColumn(#[source] deadpool_postgres::tokio_postgres::Error),

    #[error("there are no records in this table")]
    NoRecords,
}

/// A result type with [`Error`] as its error type.
pub type Result<T> = std::result::Result<T, Error>;
