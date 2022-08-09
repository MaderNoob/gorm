//! An orm that is simple to use and prevents runtime errors by using rust's rich type system to
//! enforce sql logic at compile time.
//!
//! # Usage
//!
//! The core of this crate is the [`Table`] derive macro, which you can derive on your rust structs to
//! tell the orm that they represent tables in your database.
//!
//! # Example
//! ```rust
//! #[derive(Debug, Table)]
//! pub struct Person {
//!     id: i32,
//!     name: String,
//!     age: i32,
//! 
//!     #[table(foreign_key = "School")]
//!     school_id: i32,
//! }
//! 
//! #[derive(Debug, Table)]
//! pub struct School {
//!     id: i32,
//!     name: String,
//! }
//!
//! struct MyMigration;
//! migration!{ MyMigration => school, person }
//!
//! let pool = DatabaseConnectionPool::connect("postgres://postgres:postgres@localhost/some_database")
//!     .await?;
//!
//! MyMigration::down(&pool).await?;
//! MyMigration::up(&pool).await?;
//!
//! let school_id = school::new {
//!     name: "Stanford",
//! }
//! .insert_returning_value(returning!(school::id), &pool)
//! .await?;
//!
//! person::new {
//!     name: "James",
//!     age: &35,
//!     school_id,
//! }
//! .insert(&pool)
//! .await?;
//!
//! #[derive(FromQueryResult)]
//! struct PersonNameAndSchoolName {
//!     person_name: String,
//!     school_name: String,
//! }
//! let person_and_school_names = person::table
//!     .inner_join(school::table)
//!     .find()
//!     .select(select_values!(person::name as person_name, school::name as school_name))
//!     .load_all::<PersonNameAndSchoolName>(&pool)
//!     .await?;
//!
//! struct AgeSumOfSchool {
//!     school_name: String,
//!     age_sum: i64,
//! }
//! let age_sum_of_each_school_from_highest_to_lowest = person::table
//!     .inner_join(school::table)
//!     .find()
//!     .select(select_values!(school::name as school_name, person::age.sum() as age_sum))
//!     .group_by(school::id)
//!     .order_by_selected_value_descending(selected_value_to_order_by!(age_sum))
//!     .load_all::<AgeSumOfSchool>(&pool)
//!     .await?;
//!
//! let old_enough_people_ids = person::table
//!     .find()
//!     .filter(person::age.greater_equals(20))
//!     .select(select_values!(person::id))
//!     .load_all_values(&pool)
//!     .await?;
//!
//! ```
//!
//! [`Table`]: gorm_macros::Table

#![feature(auto_traits)]
#![feature(negative_impls)]

mod error;
pub mod execution;
pub mod sql;
pub mod statements;
pub mod util;

pub use async_trait::async_trait;
pub use deadpool_postgres::tokio_postgres;
pub use error::*;
pub use futures;
pub use gorm_macros::{
    migration, returning, select_values, selected_value_to_order_by, FromQueryResult, Table,
};
pub use rust_decimal::Decimal;
pub use sql::{FromQueryResult, Table};
