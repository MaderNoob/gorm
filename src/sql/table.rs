use super::FieldNameCharsConsListItem;
use crate::{
    sql::{FieldsConsListItem, IntoSqlType, SqlType},
    statements::{
        CreateTableStatement, DropTableStatement, EmptyDeleteStatement, EmptyUpdateStatement,
    },
    util::TypesEqual,
};

/// A table in the database.
pub trait Table: Sized + 'static {
    /// A type used to identify the fields of the table.
    type Fields: FieldsConsListItem;

    /// Information about each field in the table.
    const FIELDS: &'static [TableField];

    /// Information about each unique constraint in the table.
    const UNIQUE_CONSTRAINTS: &'static [TableUniqueConstraint];

    /// The name of the table as a string.
    const TABLE_NAME: &'static str;

    /// The `id` column of this table.
    type IdColumn: Column;
}

/// Information about a field of a table struct.
pub struct TableField {
    /// The name of the field.
    pub name: &'static str,

    /// Is this field the primary key?
    pub is_primary_key: bool,

    /// Does this field have a unique constraint on it?
    pub is_unique: bool,

    /// The name of the table which this field has a foreign key constraint to,
    /// if any.
    pub foreign_key_to_table_name: Option<&'static str>,

    /// The sql string representation of this field's sql type.
    pub sql_type_name: &'static str,

    /// Is this field nullable?
    pub is_null: bool,
}

/// Information about a unique constraint of a table struct.
pub struct TableUniqueConstraint {
    /// The fields of this unique constraint
    pub fields: &'static [&'static str],
}

/// A column of a table
pub trait Column {
    /// The name of the column as a string.
    const COLUMN_NAME: &'static str;

    /// A type used to identify the name of the column.
    type ColumnName: FieldNameCharsConsListItem;

    /// The table which this column belongs to.
    type Table: Table;

    /// The sql type of this column.
    type SqlType: SqlType;

    /// The rust type of this column.
    type RustType: IntoSqlType<SqlType = Self::SqlType>;
}

/// A marker trait indicating that some column is a foreign key to another
/// table.
pub trait ColumnIsForeignKey<T: Table>: Column
where
    (
        <Self::SqlType as SqlType>::NonNullSqlType,
        <T::IdColumn as Column>::SqlType,
    ): TypesEqual,
{
}

/// A trait for representing a table marker, which is an empty struct type which
/// is used to reference some table.
pub trait TableMarker: Sized + 'static {
    /// The table referenced by this table marker.
    type Table: Table;

    /// Returns a create table statement for this table
    fn create(self) -> CreateTableStatement<Self::Table> {
        CreateTableStatement::new()
    }

    /// Returns a drop table statement for this table
    fn drop(self) -> DropTableStatement<Self::Table> {
        DropTableStatement::new()
    }

    /// Returns a delete statement for this table
    fn delete(self) -> EmptyDeleteStatement<Self::Table> {
        EmptyDeleteStatement::new()
    }

    /// Returns an update statement for this table
    fn update(self) -> EmptyUpdateStatement<Self::Table> {
        EmptyUpdateStatement::new()
    }
}

/// Indicates that some table has a foreign key to some other table
pub trait TableHasOneForeignKey<T: Table>: Table
where
    (
        <<Self::ForeignKeyColumn as Column>::SqlType as SqlType>::NonNullSqlType,
        <T::IdColumn as Column>::SqlType,
    ): TypesEqual,
{
    /// The column which contains the foreign key.
    type ForeignKeyColumn: Column<Table = Self> + ColumnIsForeignKey<T>;
}
