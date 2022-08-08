use crate::{error::*, execution::SqlStatementExecutor};

/// A migration which can be used to create and drop many tables conveniently.
#[async_trait::async_trait]
pub trait Migration {
    /// Creates the tables of this migration.
    async fn up<E: SqlStatementExecutor>(executor: &E) -> Result<()>;

    /// Drops the tables of this migration.
    async fn down<E: SqlStatementExecutor>(executor: &E) -> Result<()>;
}
