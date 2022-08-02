#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(generic_associated_types)]

mod error;
pub mod execution;
pub mod sql;
pub mod statements;
mod util;

pub use error::*;
pub use gorm_macros::{select_values, FromQueryResult, Table};
pub use paste;
pub use sql::{FromQueryResult, Table};
pub use tokio_postgres;
pub use rust_decimal::Decimal;
pub use util::*;
