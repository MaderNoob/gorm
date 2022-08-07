use super::{
    AverageableSqlType, OrderableSqlType, SelectableTables, SqlAverage, SqlCount, SqlCountRows,
    SqlExpression, SqlMax, SqlMin, SqlSum, SummableSqlType,
};

macro_rules! define_one_expr_arg_sql_function {
    {$fn_name: ident, $expr_type_name: ident $(,$where_path: path : $where_condition: path)?} => {
        pub fn $fn_name<S: SelectableTables, E: SqlExpression<S>>(expr: E) -> $expr_type_name<S, E>
            $(
                where $where_path: $where_condition
            )?
        {
            $expr_type_name::new(expr)
        }
    };
}

define_one_expr_arg_sql_function! {count, SqlCount}
define_one_expr_arg_sql_function! {average, SqlAverage, E::SqlType: AverageableSqlType}
define_one_expr_arg_sql_function! {sum, SqlSum, E::SqlType: SummableSqlType}
define_one_expr_arg_sql_function! {max, SqlMax, E::SqlType: OrderableSqlType}
define_one_expr_arg_sql_function! {min, SqlMin, E::SqlType: OrderableSqlType}

pub fn count_rows() -> SqlCountRows {
    SqlCountRows
}
