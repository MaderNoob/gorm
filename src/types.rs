/// An sql type.
pub trait SqlType {
    type RustType;
    const SQL_NAME: &'static str;
}

/// An sql type.
pub trait SqlSerialType {
    type RustType;
    const SQL_NAME: &'static str;
}

/// A trait used to convert a rust type to its sql type.
pub trait IntoSqlType {
    type SqlType: SqlType;
}

/// A trait used to convert a rust type to its sql serial type.
pub trait IntoSqlSerialType {
    type SqlSerialType: SqlSerialType;
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
            impl SqlSerialType for $sql_serial_type_name{
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
    [serial] Serial16, "smallserial" => i16 ,
    [serial] Serial32, "serial" => i32,
    [serial] Serial64, "bigserial" => i64,
}
