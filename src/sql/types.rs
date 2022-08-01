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

macro_rules! define_generic_sql_type {
    {
        $sql_type_name: ident, $sql_name: expr => $rust_type: ty
    } => {
        pub struct $sql_type_name;
        impl SqlType for $sql_type_name{
            type RustType = $rust_type;
            const SQL_NAME:&'static str = $sql_name;
        }
        impl IntoSqlType for $rust_type {
            type SqlType = $sql_type_name;
        }
    };
}

macro_rules! define_sql_type {
    {
        $sql_type_name: ident, $sql_name: expr => $rust_type: ty
    } => {
        define_generic_sql_type!{$sql_type_name, $sql_name => $rust_type}
    };
    {
        serial $sql_serial_type_name: ident, $serial_sql_name: expr => $serial_rust_type: ty
    } => {
        pub struct $sql_serial_type_name;
        impl SqlSerialType for $sql_serial_type_name{
            type RustType = $serial_rust_type;
            const SQL_NAME:&'static str = $serial_sql_name;
        }
        impl IntoSqlSerialType for $serial_rust_type {
            type SqlSerialType = $sql_serial_type_name;
        }
    };
    {
        orderable $sql_type_name: ident, $sql_name: expr => $rust_type: ty
    } => {
        define_generic_sql_type!{$sql_type_name, $sql_name => $rust_type}
        impl OrderableSqlType for $sql_type_name {}
    };
}

pub trait OrderableSqlType {}

define_sql_type! { SqlBool, "boolean" => bool }
define_sql_type! { orderable SqlI16, "smallint" => i16 }
define_sql_type! { orderable SqlI32, "integer" => i32 }
define_sql_type! { orderable SqlI64, "bigint" => i64 }
define_sql_type! { orderable SqlF32, "real" => f32 }
define_sql_type! { orderable SqlF64, "double precision" => f64 }
define_sql_type! { SqlText, "text" => String }
define_sql_type! { serial Serial16, "smallserial" => i16  }
define_sql_type! { serial Serial32, "serial" => i32 }
define_sql_type! { serial Serial64, "bigserial" => i64 }

impl<'a> IntoSqlType for &'a str {
    type SqlType = SqlText;
}
