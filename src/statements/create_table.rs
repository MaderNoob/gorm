use std::{fmt::Write, marker::PhantomData};

use crate::{
    sql::{ParameterBinder, Table, TableField, TableUniqueConstraint},
    statements::SqlStatement,
    util::TypedConsListNil,
};

/// An sql create table statement
///
/// This statement can be created by calling the [`TableMarker::create`]
/// function.
///
/// [`TableMarker::create`]: crate::sql::TableMarker::create
pub struct CreateTableStatement<T: Table>(PhantomData<T>);
impl<T: Table> CreateTableStatement<T> {
    /// Creates a new sql create table statement.
    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// Only create the table if a table with such a name doesn't already exist,
    /// and if such a table already exists, do nothing.
    pub fn if_not_exists(self) -> CreateTableIfNotExistsStatement<T> {
        CreateTableIfNotExistsStatement(PhantomData)
    }
}

impl<T: Table> SqlStatement for CreateTableStatement<T> {
    type OutputFields = TypedConsListNil;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "CREATE TABLE {} (", T::TABLE_NAME,)?;
        generate_create_table_content_sql_string(T::FIELDS, T::UNIQUE_CONSTRAINTS, f);
        f.push(')');

        Ok(())
    }
}

/// An sql create table if not exists statement.
///
/// This statement only creates the table if a table with such a name doesn't
/// already exist, otherwise it does nothing.
///
/// This statement can be created by calling the
/// [`CreateTableStatement::if_not_exists`] function.
pub struct CreateTableIfNotExistsStatement<T: Table>(PhantomData<T>);
impl<T: Table> SqlStatement for CreateTableIfNotExistsStatement<T> {
    type OutputFields = TypedConsListNil;

    fn write_sql_string<'s, 'a>(
        &'s self,
        f: &mut String,
        _parameter_binder: &mut ParameterBinder<'a>,
    ) -> std::fmt::Result
    where
        's: 'a,
    {
        write!(f, "CREATE TABLE IF NOT EXISTS {} (", T::TABLE_NAME,)?;
        generate_create_table_content_sql_string(T::FIELDS, T::UNIQUE_CONSTRAINTS, f);
        f.push(')');

        Ok(())
    }
}

/// Generates the an sql string representing the content of this table,
/// including columns, foreign keys and unique constraints.
fn generate_create_table_content_sql_string(
    fields: &[TableField],
    unique_constraints: &[TableUniqueConstraint],
    write_fields_string_into: &mut String,
) {
    for field_info in fields {
        write_fields_string_into.push_str(field_info.name);
        write_fields_string_into.push(' ');
        write_fields_string_into.push_str(field_info.sql_type_name);
        if field_info.is_primary_key {
            write_fields_string_into.push_str(" PRIMARY KEY");
        } else if !field_info.is_null {
            write_fields_string_into.push_str(" NOT NULL");
        }

        if let Some(foreign_key_to_table_name) = field_info.foreign_key_to_table_name {
            write_fields_string_into.push_str(" REFERENCES ");
            write_fields_string_into.push('"');
            write_fields_string_into.push_str(foreign_key_to_table_name);
            write_fields_string_into.push('"');
        }

        write_fields_string_into.push(',');
    }

    for unique_constraint in unique_constraints {
        write_fields_string_into.push_str("UNIQUE(");
        for field in unique_constraint.fields {
            write_fields_string_into.push_str(field);
            write_fields_string_into.push(',');
        }

        // remove the trailing comma
        if write_fields_string_into.ends_with(',') {
            write_fields_string_into.pop();
        }

        // close the parentheses and add a comma for the next unique constraint.
        write_fields_string_into.push_str("),");
    }

    // remove the trailing comma
    if write_fields_string_into.ends_with(',') {
        write_fields_string_into.pop();
    }
}
