mod from_query_result;
mod migration;
mod select_values;
mod selected_value_to_order_by;
mod table;
mod util;

use proc_macro::TokenStream;
use util::generate_field_name_cons_list_type;

/// A macro which allows selecting custom values from a query.
///
/// The input to this macro should be a list of comma seperated expression to select from the
/// query.
///
/// If one of the selected expressions is not just a column of some table, its field name must
/// be explicitly specified by adding `as <field_name>` at the end of the expression. Note that
/// sometimes you will need to add parentheses around the expression in order for the `as
/// <field_name>` to be detected by this macro.
///
/// # Example
///
/// ```rust
/// #[derive(Table)]
/// struct Person {
///     id: i32,
///     age: i32,
///     name: String,
/// }
///
/// let _ = person::table
///     .find()
///     .select(select_values!(person::name, person::id.add(person::age) as id_plus_age))
///     .load::<...>(...)
///     .await?;
/// ```
#[proc_macro]
pub fn select_values(input_tokens: TokenStream) -> TokenStream {
    select_values::select_values(input_tokens)
}

/// This macro is just another name for the [`select_values!`] macro, for usage information check
/// out the documentation on that macro.
#[proc_macro]
pub fn returning(input_tokens: TokenStream) -> TokenStream {
    select_values::select_values(input_tokens)
}

/// Implements the `Migration` trait for some struct, given the tables that it should manage.
///
/// # Example
///
/// ```rust
/// struct MyMigration;
/// migration! { MyMigration =>  school, person}
///
/// // This is how you can use the migration. The up function creates all the tables in the correct
/// // order, and the down functions drops them. 
/// MyMigration::up(...).await?;
/// MyMigration::down(...).await?;
///
/// #[derive(Table)]
/// struct Person {
///     id: i32,
///     name: String,
///     
///     #[table(foreign_key = "School")]
///     school_id: i32,
/// }
///
/// #[derive(Table)]
/// struct School {
///     id: i32,
///     name: String,
/// }
/// ```
///
/// Please note that the order of tables that are provided to this macro must be in the correct
/// order by which the tables should be created. For example in the example above if we were to put
/// the `person` table before the `school` table we would get an error because `person` has a
/// foreign key to `school`.
#[proc_macro]
pub fn migration(input_tokens: TokenStream) -> TokenStream {
    migration::migration(input_tokens)
}

/// Implements the `FromQueryResult` trait for some struct.
///
/// This allows the struct to be loaded from query results, for example from `SELECT` statements.
///
/// # Example
/// ```rust
/// #[derive(FromQueryResult)]
/// struct PersonNameAndSchoolName {
///     person_name: String,
///     school_name: String,
/// }
///
/// let _ = person::table.inner_join(school::table)
///     .find()
///     .select(select_values!(person::name as person_name, school::name as school_name))
///     .load_all::<PersonNameAndSchoolName>(...)
///     .await?;
///
/// #[derive(Table)]
/// struct Person {
///     id: i32,
///     name: String,
///     
///     #[table(foreign_key = "School")]
///     school_id: i32,
/// }
///
/// #[derive(Table)]
/// struct School {
///     id: i32,
///     name: String,
/// }
/// ```
#[proc_macro_derive(FromQueryResult)]
pub fn from_query_result(input_tokens: TokenStream) -> TokenStream {
    from_query_result::from_query_result(input_tokens)
}

/// Given a field name as an ident, returns a field name typed cons list type.
#[proc_macro]
pub fn create_field_name_cons_list(item: TokenStream) -> TokenStream {
    generate_field_name_cons_list_type(&item.to_string()).into()
}

/// Implements the `Table` trait for the some struct, and creates a table module for it containing
/// some useful items which allow performing operations on this table.
///
/// # Example
/// ```rust
/// #[derive(Table)]
/// struct Person {
///     id: i32,
///     name: String,
///     
///     #[table(foreign_key = "School")]
///     school_id: i32,
/// }
///
/// #[derive(Table)]
/// struct School {
///     id: i32,
///     name: String,
/// }
/// ```
///
/// The `Table` macro, besides implementing the `Table` trait for the provided struct, will also
/// create a module which has the same name as the struct, but converted to `snake_case`. This
/// module will contain useful items which allow peforing operations on the table.
///
/// For example, for the `Person` struct, a module named `person` will be created, containing the 
/// following items:
///
///  - A struct called `new` (`person::new`), which contains all fields of a person other than its
///  id. The `new` struct implements the `Insertable` trait which allows inserting it to the
///  database. The fields of this struct use the [`std::borrow::Borrow`] trait to allow for
///  providing things that the actual type can be borrowed us. For example when creating a school
///  using `school::new`, we can provide a value of type `&str` for the `name` field.
///  
///  - A struct called `new_with_id`, same as the `new` struct but allows specifying a value for
///  the id field.
///
///  - An empty struct called `table` (`person::table`), which implements the `TableMarker` trait. This
///  struct allows you to perform operations on the table like `create`, `drop`, `delete`, `find`,
///  `inner_join`.
///
///  - An empty struct called `all` (`person::all`) which implements the `SelectedValues` trait and
///  allows selecting all fields of this table in functions which require selecting custom values.
///
///  - An empty struct for each column in the table. For example in the above example, the created structs
///  will be `person::id`, `person::name` and `person::age`. Each of these structs implement the
///  `SqlExpression` trait.
///
/// # Foreign Keys
///
/// Foreign keys can be implemented as shown in the example above using the 
/// `#[table(foreign_key = "...")]` attribute, and specifying the table struct's name.
///
/// Foreign keys allow you to perform joins on the tables, and we can then perform select 
/// queries on the joined tables, for example, for the above snippet we can do the following:
/// ```rust
/// let _ = person::table.inner_join(school::table)
///     .find()
///     .filter(school::name.equals("Stanford"))
///     .load::<Person>(...)
///     .await?;
/// ```
///
/// Foreign keys can also be optional, for example:
/// ```rust
/// #[derive(Table)]
/// struct MaybeStudent {
///     id: i32,
///     
///     #[table(foreign_key = "School")]
///     school_id: Option<i32>,
/// }
/// ```
///
/// The inner join of the table `MaybeStudent` with the table `School` will then only return the
/// students who's `school_id` is not `None`.
///
/// Please note that the type of the foreign key field must match the type of the referenced
/// table's `id` field.
#[proc_macro_derive(Table, attributes(table))]
pub fn table(input_tokens: TokenStream) -> TokenStream {
    table::table(input_tokens)
}

/// This macro provides a way to order the results of a query by a value selected using the
/// `select_values` macro.
///
/// # Example
/// ```rust
/// let _ = person::table
///     .find()
///     .select(select_values!(
///         person::age.multiply(person::id) as some_name
///     ))
///     .order_by_selected_value_descending(selected_value_to_order_by!(some_name))
///     .load_all_values(...)
///     .await?;
///
/// #[derive(Table)]
/// struct Person {
///     id: i32,
///     name: String,
/// }
/// ```
#[proc_macro]
pub fn selected_value_to_order_by(input_tokens: TokenStream) -> TokenStream {
    selected_value_to_order_by::selected_value_to_order_by(input_tokens)
}
