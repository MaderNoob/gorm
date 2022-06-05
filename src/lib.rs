use std::marker::PhantomData;

pub use gorm_macros::Table;
pub use sqlx;

/// A typed cons list of fields.
pub trait FieldsConsListItem {}

/// A cons item of a typed cons list of fields.
pub struct FieldsConsListCons<
    FieldName: FieldNameCharsConsListItem,
    FieldType,
    Next: FieldsConsListItem,
>(
    PhantomData<FieldName>,
    PhantomData<FieldType>,
    PhantomData<Next>,
);
impl<FieldName: FieldNameCharsConsListItem, FieldType: IntoSqlType, Next: FieldsConsListItem>
    FieldsConsListItem for FieldsConsListCons<FieldName, FieldType, Next>
{
}

/// A typed cons list of field name characters.
pub trait FieldNameCharsConsListItem {}

/// A cons item of a typed cons list of field name characters.
pub struct FieldNameCharsConsListCons<const CHAR: char, Next: FieldNameCharsConsListItem>(
    PhantomData<Next>,
);
impl<const CHAR: char, Next: FieldNameCharsConsListItem> FieldNameCharsConsListItem
    for FieldNameCharsConsListCons<CHAR, Next>
{
}

/// A nil item of a types cons list.
pub struct TypedConsListNil;
impl FieldsConsListItem for TypedConsListNil {}
impl FieldNameCharsConsListItem for TypedConsListNil {}

/// A table in the database.
pub trait Table: Sized {
    type Fields: FieldsConsListItem;
    const FIELDS: &'static [TableField];
    const TABLE_NAME: &'static str;

    /// Returns a create table statement for this table
    fn create_table_statement() -> CreateTableStatement<Self> {
        CreateTableStatement(PhantomData)
    }

    /// Returns a drop table statement for this table
    fn drop_table_statement() -> DropTableStatement<Self> {
        DropTableStatement(PhantomData)
    }
}

/// Information about a field of a table struct
pub struct TableField {
    pub name: &'static str,
    pub is_primary_key: bool,
    pub sql_type_name: &'static str,
}

/// An sql create table statement
pub struct CreateTableStatement<T: Table>(PhantomData<T>);
impl<T: Table> CreateTableStatement<T> {
    /// Do not throw an error if a table with that name already exists.
    pub fn if_not_exists(self) -> CreateTableIfNotExistsStatement<T> {
        CreateTableIfNotExistsStatement(PhantomData)
    }
}

impl<T: Table> SqlStatement for CreateTableStatement<T> {
    fn to_sql_string(self) -> String {
        format!(
            "CREATE TABLE {} ({})",
            T::TABLE_NAME,
            generate_create_table_columns_sql_string(T::FIELDS)
        )
    }
}

/// An sql create table if not exists statement
pub struct CreateTableIfNotExistsStatement<T: Table>(PhantomData<T>);
impl<T: Table> SqlStatement for CreateTableIfNotExistsStatement<T> {
    fn to_sql_string(self) -> String {
        format!(
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
        }
        fields_string.push(',');
    }

    // remove the trailing comma
    if fields_string.ends_with(',') {
        fields_string.pop();
    }

    fields_string
}

/// An sql drop table statement
pub struct DropTableStatement<T: Table>(PhantomData<T>);
impl<T: Table> DropTableStatement<T> {
    /// Do not throw an error if the table does not exist.
    pub fn if_exists(self) -> DropTableIfExistsStatement<T> {
        DropTableIfExistsStatement(PhantomData)
    }
}
impl<T: Table> SqlStatement for DropTableStatement<T> {
    fn to_sql_string(self) -> String {
        format!(
            "DROP TABLE {}",
            T::TABLE_NAME
        )
    }
}

/// An sql drop table if exists statement
pub struct DropTableIfExistsStatement<T: Table>(PhantomData<T>);
impl<T: Table> SqlStatement for DropTableIfExistsStatement<T> {
    fn to_sql_string(self) -> String {
        format!(
            "DROP TABLE IF EXISTS {}",
            T::TABLE_NAME
        )
    }
}

/// An sql statement which can be executed on the database.
pub trait SqlStatement {
    /// Converts the sql statement to an sql string which can be executed on the database.
    fn to_sql_string(self) -> String;
}

/// An sql type.
pub trait SqlType {
    type RustType;
    const SQL_NAME: &'static str;
}

/// A trait used to convert a rust type to its sql type.
pub trait IntoSqlType {
    type SqlType: SqlType;
}

/// A trait used to convert a rust type to its sql serial type.
pub trait IntoSqlSerialType {
    type SqlSerialType: SqlType;
}

macro_rules! define_sql_types {
    {
        $( $sql_type_name: ident, $sql_name: expr => $rust_type: ty, )+
        $( [serial] $sql_serial_type_name: ident, $serial_sql_name: expr => $serial_rust_type: ty, )+
    } => {
       $(
            pub struct $sql_type_name;
            impl SqlType for $sql_type_name{
                type RustType = $rust_type;
                const SQL_NAME:&'static str = $sql_name;
            }
            impl IntoSqlType for $rust_type {
                type SqlType = $sql_type_name;
            }
        )+
       $(
            pub struct $sql_serial_type_name;
            impl SqlType for $sql_serial_type_name{
                type RustType = $serial_rust_type;
                const SQL_NAME:&'static str = $serial_sql_name;
            }
            impl IntoSqlSerialType for $serial_rust_type {
                type SqlSerialType = $sql_serial_type_name;
            }
        )+
    };
}

define_sql_types! {
    SqlBool, "boolean" => bool,
    SqlI16, "smallint" => i16,
    SqlI32, "integer" => i32,
    SqlI64, "bigint" => i64,
    SqlF32, "real" => f32,
    SqlF64, "double precision" => f64,
    Text, "text" => String,
    Binary, "bytea" => Vec<u8>,
    [serial] Serial16, "smallserial" => i16 ,
    [serial] Serial32, "serial" => i32,
    [serial] Serial64, "bigserial" => i64,
}
