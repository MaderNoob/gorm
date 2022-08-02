use rust_decimal::Decimal;

/// An sql type.
pub trait SqlType {
    type RustType: IntoSqlType<SqlType = Self>;
    const SQL_NAME: &'static str;
}

/// An sql type.
pub trait SqlSerialType {
    type RustType: IntoSqlSerialType<SqlSerialType = Self>;
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
}

define_sql_type! { SqlBool, "boolean" => bool }
define_sql_type! { SqlI16, "smallint" => i16 }
define_sql_type! { SqlI32, "integer" => i32 }
define_sql_type! { SqlI64, "bigint" => i64 }
define_sql_type! { SqlF32, "real" => f32 }
define_sql_type! { SqlF64, "double precision" => f64 }
define_sql_type! { SqlNumeric, "numeric" => Decimal }
define_sql_type! { SqlText, "text" => String }
define_sql_type! { serial Serial16, "smallserial" => i16  }
define_sql_type! { serial Serial32, "serial" => i32 }
define_sql_type! { serial Serial64, "bigserial" => i64 }

impl<'a> IntoSqlType for &'a str {
    type SqlType = SqlText;
}

pub trait OrderableSqlType {}

macro_rules! mark_sql_types {
    {$marker_trait: ty => $($t:ty),*} => {
        $(
            impl $marker_trait for $t {}
        )*
    };
}

mark_sql_types!{OrderableSqlType => SqlI16, SqlI32, SqlI64, SqlF32, SqlF64, SqlNumeric, Serial16, Serial32, Serial64, SqlText}

pub trait AverageableSqlType {
    type OutputSqlType: SqlType;
}

pub trait SummableSqlType: SqlAdd<Self> + Sized {
    type OutputSqlType: SqlType;
}

macro_rules! mark_sql_types_with_output_type {
    {$marker_trait: ty => $($t:ty : $output_ty:ty),*} => {
        $(
            impl $marker_trait for $t {
                type OutputSqlType = $output_ty;
            }
        )*
    };
}

mark_sql_types_with_output_type!{AverageableSqlType => 
    SqlI16: SqlNumeric, 
    SqlI32: SqlNumeric, 
    SqlI64: SqlNumeric, 
    SqlF32: SqlF64, 
    SqlF64: SqlF64, 
    SqlNumeric: SqlNumeric, 
    Serial16: SqlNumeric,
    Serial32: SqlNumeric,
    Serial64: SqlNumeric
}

mark_sql_types_with_output_type!{SummableSqlType => 
    SqlI16: SqlI64, 
    SqlI32: SqlI64, 
    SqlI64: SqlNumeric, 
    SqlF32: SqlF32, 
    SqlF64: SqlF64, 
    SqlNumeric: SqlNumeric, 
    Serial16: SqlI64,
    Serial32: SqlI64,
    Serial64: SqlNumeric
}


macro_rules! mark_sql_types_with_rhs {
    ($marker_trait: ident => $($lhs: ident : ($($rhs: ident),*)),*) => {
        $(
            $(
                impl $marker_trait<$rhs> for $lhs {}
            )*
        )*
    };
}

pub trait SqlAdd<Rhs> {}
pub trait SqlSubtract<Rhs> {}
pub trait SqlMultiply<Rhs> {}
pub trait SqlDivide<Rhs> {}

macro_rules! mark_all_number_types_with_rhs {
    ($marker_trait: ident) => {
        mark_sql_types_with_rhs!{ $marker_trait =>
            SqlI16: (SqlI16, Serial16),
            SqlI32: (SqlI32, Serial32),
            SqlI64: (SqlI64, Serial64),
            SqlF32: (SqlF32),
            SqlF64: (SqlF64),
            SqlNumeric: (SqlNumeric),
            Serial16: (Serial16, SqlI16),
            Serial32: (Serial32, SqlI32),
            Serial64: (Serial64, SqlI64)
        }
    };
}

mark_all_number_types_with_rhs!{SqlAdd}
mark_all_number_types_with_rhs!{SqlSubtract}
mark_all_number_types_with_rhs!{SqlMultiply}
mark_all_number_types_with_rhs!{SqlDivide}
