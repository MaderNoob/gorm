use crate::{
    fields_list::FieldsConsListItem,
    statements::{CreateTableStatement, DropTableStatement, InnerJoined, SelectFrom},
    types::SqlType,
};

/// A table in the database.
pub trait Table: Sized {
    type Fields: FieldsConsListItem;
    const FIELDS: &'static [TableField];
    const TABLE_NAME: &'static str;
    type IdColumn: Column;
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

/// A trait for representing a table marker, which is an empty type which is used to reference a
/// table.
pub trait TableMarker: Sized {
    type Table: Table;

    /// Returns a create table statement for this table
    fn create_table(self) -> CreateTableStatement<Self::Table> {
        CreateTableStatement::new()
    }

    /// Returns a drop table statement for this table
    fn drop_table(self) -> DropTableStatement<Self::Table> {
        DropTableStatement::new()
    }
}

/// Indicates that some table has a foreign key to some other table
pub trait HasForeignKey<T: Table>: Table {
    type ForeignKeyColumn: Column<SqlType = <T::IdColumn as Column>::SqlType>;
}
