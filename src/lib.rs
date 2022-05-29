pub trait Table {
    fn fields() -> &'static [FieldInfo];
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldInfo {
    pub name: &'static str,
    pub ty: SqlType,
}
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SqlType {
    Bool,
    I16,
    I32,
    I64,
    F32,
    F64,
    Serial16,
    Serial32,
    Serial64,
    Text,
    Binary,
    // Timestamps
}
pub trait IntoSqlType{
    const SQL_TYPE: SqlType;
}
macro_rules! impl_into_sql_type {
    {$for: ty, $sql_ty: ident} => {
        impl IntoSqlType for $for {
            const SQL_TYPE:SqlType = SqlType::$sql_ty;
        }
    };
}
impl_into_sql_type!{String, Text}
impl_into_sql_type!{bool, Bool}
impl_into_sql_type!{i16, I16}
impl_into_sql_type!{i32, I32}
impl_into_sql_type!{i64, I64}

