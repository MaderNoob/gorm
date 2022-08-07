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
