mod condition;
mod create_table;
mod drop_table;
mod select;
mod select_from;

use crate::{
    bound_parameters::ParameterBinder, error::*, fields_list::TypedConsListNil,
    from_query_result::FromQueryResult, util::TypesNotEqual, ExecuteResult,
};
use std::fmt::Display;

use async_trait::async_trait;
pub use create_table::*;
pub use drop_table::*;
pub use select::*;
pub use select_from::*;

use crate::{execution::SqlStatementExecutor, fields_list::FieldsConsListItem};

/// An sql statement which can be executed by a database.
pub trait SqlStatement: Sized + 'static {
    /// The fields of the output of this statement.
    type OutputFields: FieldsConsListItem;

    /// Writes the sql statement as an sql string which can be executed by a database.
    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;
}

#[async_trait]
pub trait ExecuteSqlStatment: SqlStatement {
    /// Executes the query on the given executor
    async fn execute(
        self,
        on: &(impl SqlStatementExecutor + Send + Sync),
    ) -> Result<ExecuteResult> {
        on.execute(self).await
    }

    async fn load_one<O: FromQueryResult>(
        self,
        on: &(impl SqlStatementExecutor + Send + Sync),
    ) -> Result<O> {
        on.load_one(self).await
    }

    async fn load_optional<O: FromQueryResult>(
        self,
        on: &(impl SqlStatementExecutor + Send + Sync),
    ) -> Result<Option<O>> {
        on.load_optional(self).await
    }
}

#[async_trait]
impl<S: SqlStatement> ExecuteSqlStatment for S {}
