use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error")]
    DatabaseError(
        #[from]
        #[source]
        sqlx::Error,
    ),
}

pub type Result<T> = std::result::Result<T, Error>;
