use crate::{fields_list::FieldsConsListItem, statements::{CreateTableStatement, DropTableStatement, FindStatement}, types::SqlType};

/// A table in the database.
pub trait Table: Sized {
    type Fields: FieldsConsListItem;
    const FIELDS: &'static [TableField];
    const TABLE_NAME: &'static str;

    /// Returns a create table statement for this table
    fn create_table() -> CreateTableStatement<Self> {
        CreateTableStatement::new()
    }

    /// Returns a drop table statement for this table
    fn drop_table() -> DropTableStatement<Self> {
        DropTableStatement::new()
    }

    /// Find records in the table
    fn find() -> FindStatement<Self> {
        FindStatement::new()
    }
}

/// Information about a field of a table struct
pub struct TableField {
    pub name: &'static str,
    pub is_primary_key: bool,
    pub sql_type_name: &'static str,
}

/// A column of a table
pub trait Column {
    const COLUMN_NAME: &'static str;
    type Table: Table;
    type SqlType: SqlType;
    type RustType;
}
