//! An orm that is simple to use and prevents runtime errors by using rust's rich type system to
//! enforce sql logic correctness at compile time.
//!
//! # Usage
//!
//! This crate provdies the `Table` derive macro, which you can derive on your rust structs to
//! tell the orm that they represent tables in your database, for example:
//!
//! ```rust
//! #[derive(Table)]
//! pub struct Person {
//!     id: i32,
//!     name: String,
//!     age: i32,
//! }
//! ```
//!
//! The `Table` derive macro will generate a module with the same name as the struct converted to
//! `snake_case`. In the above example, a module named `person` will be created. 
//!
//! This module contains a bunch of items which allow you to perform operations on the table:
//!  - A struct called `new` (`person::new`), which contains all fields of a person other than its
//!  id. The `new` struct implements the [`Insertable`] trait which allows inserting it to the
//!  database.
//!  - A struct called `new_with_id`, same as the `new` struct but allows specifying a value for
//!  the id field.
//!  - A struct called `table` (`person::table`), which implements the [`TableMarker`] trait. This
//!  struct allows you to perform operations on the table like `create`, `drop`, `delete`, `find`,
//!  `inner_join`.
//!  - A struct called `all` (`person::all`) which implements the [`SelectedValues`] trait and
//!  allows selecting all fields of this table in functions which require selecting custom values.
//!  - A struct for each column in the table. For example in the above example, the created structs
//!  will be `person::id`, `person::name` and `person::age`. Each of these structs implement the
//!  [`SqlExpression`] trait.
//!
//! # Foreign keys
//! To create a foreign key constraint from one table to another, do the following:
//!
//! ```rust
//! #[derive(Table)]
//! struct Person {
//!     id: i32,
//!     name: String,
//!     
//!     #[table(foreign_key = "School")]
//!     school_id: i32,
//! }
//!
//! #[derive(Table)]
//! struct School {
//!     id: i32,
//!     name: String,
//! }
//! ```
//!
//! Foreign key constraints allow you to perform joins on tables. For example, for the above
//! example we can do the following:
//!
//! ```rust
//! person::table.inner_join(school::table)
//! ```
//!
//! After joining tables, you can perform `SELECT` queries on them using the `find` function:
//!
//! ```rust
//! person::table.inner_join(school::table).find()
//! ```
//!
//! [`Insertable`]: crate::sql::Insertable
//! [`TableMarker`]: crate::sql::TableMarker
//! [`SelectedValues`]: crate::sql::SelectedValues
//! [`SqlExpression`]: crate::sql::SqlExpression

#![feature(auto_traits)]
#![feature(negative_impls)]

mod error;
pub mod execution;
pub mod sql;
pub mod statements;
mod util;

pub use async_trait::async_trait;
pub use deadpool_postgres::tokio_postgres;
pub use error::*;
pub use futures;
pub use gorm_macros::{
    migration, select_values, select_values as returning, selected_value_to_order_by,
    FromQueryResult, Table,
};
pub use rust_decimal::Decimal;
pub use sql::{FromQueryResult, Table};
pub use util::*;
