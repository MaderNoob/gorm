use super::{
    AverageableSqlType, OrderableSqlType, SelectableTables, SqlAverage, SqlBool, SqlCount,
    SqlCountRows, SqlExpression, SqlMax, SqlMin, SqlNot, SqlSum, SummableSqlType,
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

/// Returns an expression which evaluates to the amount of all rows returned
/// from the query.
///
/// This will be translated to `COUNT(*)` when converted to sql.
pub fn count_rows() -> SqlCountRows {
    SqlCountRows
}

/// Returns an expression which negates the value of the given boolean
/// expression.
pub fn not<S: SelectableTables, E: SqlExpression<S, SqlType = SqlBool>>(expr: E) -> SqlNot<S, E> {
    SqlNot::new(expr)
}
