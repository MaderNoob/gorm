use crate::{error::*, execution::SqlStatementExecutor};

#[async_trait::async_trait]
pub trait Migration {
    async fn up<E: SqlStatementExecutor>(executor: &E) -> Result<()>;
    async fn down<E: SqlStatementExecutor>(executor: &E) -> Result<()>;
}
