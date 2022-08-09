use deadpool_postgres::tokio_postgres::types::FromSqlOwned;

use super::{FieldNameCharsConsListItem, FieldsConsListCons, ParameterBinder, SelectedValues};
use crate::{
    error::*,
    execution::SqlStatementExecutor,
    statements::{
        ExecuteSqlStatment, InsertStatement, LoadSingleColumnSqlStatment, LoadSqlStatment,
        ReturningInsertStatement,
    },
    FromQueryResult, Table, util::{TypedConsListNil, TypesNotEqual},
};

/// A record type which can be inserted into the database.
#[async_trait::async_trait]
pub trait Insertable: Sized {
    type Table: Table;

    /// Writes the names of the values inserted by this insertable.
    /// For example, in the query:
    /// `INSERT INTO person(name,age) values('James', 29)`
    /// This represents the `name,age` part.
    fn write_value_names(&self, f: &mut String) -> std::fmt::Result;

    /// Writes the the values inserted by this insertable.
    /// For example, in the query:
    /// `INSERT INTO person(name,age) values('James', 29)`
    /// This represents the `'James',29` part.
    fn write_values<'s, 'a>(
        &'s self,
        f: &mut String,
        parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a;

    /// Inserts this record into the database.
    async fn insert(self, to: &impl SqlStatementExecutor) -> Result<()> {
        InsertStatement::new(self).execute(to).await?;

        Ok(())
    }

    /// Inserts this record into the database, returning the selected values.
    ///
    /// The values can be selected using the [`returning!`] macro.
    ///
    /// [`returning!`]: crate::returning
    async fn insert_returning<O: FromQueryResult + Send>(
        self,
        returning: impl SelectedValues<Self::Table, Fields = O::Fields> + Send + 'static,
        to: &impl SqlStatementExecutor,
    ) -> Result<O>
    where
        (O::Fields, TypedConsListNil): TypesNotEqual,
    {
        ReturningInsertStatement::new(self, returning)
            .load_one(to)
            .await
    }


    /// Inserts this record into the database, returning a single selected value.
    ///
    /// The value can be selected using the [`returning!`] macro.
    ///
    /// [`returning!`]: crate::returning
    async fn insert_returning_value<
        FieldName: FieldNameCharsConsListItem,
        FieldType: FromSqlOwned + Send,
    >(
        self,
        returning: impl SelectedValues<
            Self::Table,
            Fields = FieldsConsListCons<FieldName, FieldType, TypedConsListNil>,
        > + Send
        + 'static,
        to: &impl SqlStatementExecutor,
    ) -> Result<FieldType> {
        ReturningInsertStatement::new(self, returning)
            .load_one_value(to)
            .await
    }
}
