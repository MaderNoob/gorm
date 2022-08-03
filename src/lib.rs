#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(generic_associated_types)]
#![feature(associated_const_equality)]

mod error;
pub mod execution;
pub mod sql;
pub mod statements;
mod util;

pub use async_trait::async_trait;
pub use error::*;
pub use gorm_macros::{migration, select_values, FromQueryResult, Table};
pub use rust_decimal::Decimal;
pub use sql::{FromQueryResult, Table};
pub use tokio_postgres;
pub use util::*;
