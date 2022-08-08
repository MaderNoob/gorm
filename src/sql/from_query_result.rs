use deadpool_postgres::tokio_postgres::Row;

use crate::{error::*, sql::FieldsConsListItem, util::TypedConsListNil};

/// A type that can be parsed from an sql query result.
pub trait FromQueryResult: Sized {
    type Fields: FieldsConsListItem;

    fn from_row(row: Row) -> Result<Self>;
}

/// An empty query result.
pub struct EmptyQueryResult;
impl FromQueryResult for EmptyQueryResult {
    type Fields = TypedConsListNil;

    fn from_row(_row: Row) -> Result<Self> {
        Ok(Self)
    }
}
