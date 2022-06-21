use std::marker::PhantomData;

use crate::{table::{Table, TableField}, statements::SqlStatement};

/// An sql create table statement
pub struct CreateTableStatement<T: Table>(PhantomData<T>);
impl<T: Table> CreateTableStatement<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// Do not throw an error if a table with that name already exists.
    pub fn if_not_exists(self) -> CreateTableIfNotExistsStatement<T> {
        CreateTableIfNotExistsStatement(PhantomData)
    }
}

impl<T: Table> SqlStatement for CreateTableStatement<T> {
    fn write_sql_string(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CREATE TABLE {} ({})",
            T::TABLE_NAME,
            generate_create_table_columns_sql_string(T::FIELDS)
        )
    }
}

/// An sql create table if not exists statement
pub struct CreateTableIfNotExistsStatement<T: Table>(PhantomData<T>);
impl<T: Table> SqlStatement for CreateTableIfNotExistsStatement<T> {
    fn write_sql_string(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CREATE TABLE IF NOT EXISTS {} ({})",
            T::TABLE_NAME,
            generate_create_table_columns_sql_string(T::FIELDS)
        )
    }
}

fn generate_create_table_columns_sql_string(fields: &[TableField]) -> String {
    let mut fields_string = String::new();
    for field_info in fields {
        fields_string.push_str(&field_info.name);
        fields_string.push(' ');
        fields_string.push_str(&field_info.sql_type_name);
        if field_info.is_primary_key {
            fields_string.push_str(" PRIMARY KEY");
        } else {
            fields_string.push_str(" NOT NULL");
        }

        if let Some(foreign_key_to_table_name) = field_info.foreign_key_to_table_name{
            fields_string.push_str(" REFERENCES ");
            fields_string.push('"');
            fields_string.push_str(foreign_key_to_table_name);
            fields_string.push('"');
        }

        fields_string.push(',');
    }

    // remove the trailing comma
    if fields_string.ends_with(',') {
        fields_string.pop();
    }

    fields_string
}
