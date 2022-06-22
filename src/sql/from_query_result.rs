use sqlx::{FromRow, Row};

use crate::fields_list::{FieldsConsListItem, TypedConsListNil};

/// A type that can be built from an sql query result.
pub trait FromQueryResult<'a, R: Row>: FromRow<'a, R> {
    type Fields: FieldsConsListItem;
}

/// An empty query result.
pub struct EmptyQueryResult;
impl<'a, R: Row> FromQueryResult<'a, R> for EmptyQueryResult {
    type Fields = TypedConsListNil;
}
impl<'a, R: Row> FromRow<'a, R> for EmptyQueryResult {
    fn from_row(_row: &'a R) -> Result<Self, sqlx::Error> {
        Ok(Self)
    }
}
