use async_trait::async_trait;

use crate::{error::*, SqlStatement};

pub mod pool;
pub mod connection;

/// An executor which can execute sql statements
#[async_trait(?Send)]
pub trait SqlStatementExecutor {
    async fn execute(self, statement: impl SqlStatement) -> Result<()>;
}
