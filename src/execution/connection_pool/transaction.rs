use deadpool_postgres::Transaction;

use crate::{
    error::*,
    execution::{impl_sql_statement_executor, ExecuteResult},
    sql::FromQueryResult,
    statements::SqlStatement,
};

/// A database transaction from a connection pool.
///
/// The transaction can be commited using the
/// [`DatabaseTransactionFromPool::commit`] function.
///
/// When the transaction is dropped, if the transaction was not commited, it is
/// automatically rolled back.
pub struct DatabaseTransactionFromPool<'a> {
    pub(super) transaction: Transaction<'a>,
}

impl<'a> DatabaseTransactionFromPool<'a> {
    /// Commits the transaction to the database.
    pub async fn commit(self) -> Result<()> {
        self.transaction.commit().await?;
        Ok(())
    }
}

async fn get_raw_executor<'a, 'r>(
    database_transaction: &'r DatabaseTransactionFromPool<'a>,
) -> Result<&'r Transaction<'a>> {
    Ok(&database_transaction.transaction)
}

impl_sql_statement_executor! {DatabaseTransactionFromPool<'a>, get_raw_executor, 'a}
