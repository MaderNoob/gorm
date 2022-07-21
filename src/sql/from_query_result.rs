use tokio_postgres::Row;

use crate::{fields_list::{FieldsConsListItem, TypedConsListNil}, error::*};

/// A type that can be built from an sql query result.
pub trait FromQueryResult: Sized{
    type Fields: FieldsConsListItem;

    fn from_row(row: Row)->Result<Self>;
}

/// An empty query result.
pub struct EmptyQueryResult;
impl FromQueryResult for EmptyQueryResult {
    type Fields = TypedConsListNil;

    fn from_row(row: Row)->Result<Self> {
        Ok(Self)
    }
}

struct Person{
    name: String,
    age: u32,
}

impl FromQueryResult for Person{
    type Fields = TypedConsListNil;

    fn from_row(row: Row)->Result<Self> {
        Ok(Self{
            name: row.try_get("name").map_err(Error::FailedToGetColumn)?,
            age: row.try_get("age")?,
        })
    }
}
