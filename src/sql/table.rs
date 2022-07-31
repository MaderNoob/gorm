use crate::{
    fields_list::FieldsConsListItem,
    statements::{CreateTableStatement, DropTableStatement},
    types::{SqlType, IntoSqlType},
};

/// A table in the database.
pub trait Table: Sized + 'static {
    type Fields: FieldsConsListItem;
    const FIELDS: &'static [TableField];
    const TABLE_NAME: &'static str;
    type IdColumn: Column;
}

/// Information about a field of a table struct
pub struct TableField {
    pub name: &'static str,
    pub is_primary_key: bool,
    pub foreign_key_to_table_name: Option<&'static str>,
    pub sql_type_name: &'static str,
}

/// A column of a table
pub trait Column {
    const COLUMN_NAME: &'static str;
    type Table: Table;
    type SqlType: SqlType;
    type RustType: IntoSqlType;
}

/// A trait for representing a table marker, which is an empty type which is used to reference a
/// table.
pub trait TableMarker: Sized + 'static {
    type Table: Table;

    /// Returns a create table statement for this table
    fn create(self) -> CreateTableStatement<Self::Table> {
        CreateTableStatement::new()
    }

    /// Returns a drop table statement for this table
    fn drop(self) -> DropTableStatement<Self::Table> {
        DropTableStatement::new()
    }
}

/// Indicates that some table has a foreign key to some other table
pub trait HasForeignKey<T: Table>: Table {
    type ForeignKeyColumn: Column<SqlType = <T::IdColumn as Column>::SqlType>;
}
