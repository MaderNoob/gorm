//! To run this example you must enable to `migration_cli` feature flag, which can be done using
//! the following command:
//!
//! `cargo run --example migration_cli --features migration_cli`

mod tables;
use tables::*;

use gorm::migration_cli_main;

#[tokio::main]
async fn main(){
    migration_cli_main(CreateTablesMigration, "postgres://postgres:postgres@localhost/gorm_test").await;
}
