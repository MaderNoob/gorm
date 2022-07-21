use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error")]
    DatabaseError(
        #[from]
        #[source]
        tokio_postgres::Error,
    ),

    #[error("failed to get column from query results: {0}")]
    FailedToGetColumn(#[source] tokio_postgres::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
