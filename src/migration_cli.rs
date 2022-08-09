use crate::{execution::DatabaseConnection, sql::Migration};
use anyhow::Context;
use clap::Parser;

/// The entry point of a migration cli.
pub async fn migration_cli_main<M: Migration>(migration: M, database_connection_url: &str) {
    match migration_cli_main_async_fallible(migration, database_connection_url).await {
        Ok(()) => {
            println!("done");
        }
        Err(err) => {
            println!("error: {:?}", err);
        }
    }
}

pub async fn migration_cli_main_async_fallible<M: Migration>(
    _migration: M,
    database_connection_url: &str,
) -> anyhow::Result<()> {
    #[derive(Parser)]
    #[clap(name = "Migration command line interface")]
    #[clap(about = "Provides a command line interface for controlling a migration")]
    enum Args {
        /// Creates all the tables associated with the migration
        Up,

        /// Drops all the tables associated with the migration
        Down,
    }

    let args = Args::parse();

    let connection = DatabaseConnection::connect(database_connection_url)
        .await
        .context("failed to connect to database")?;

    match args {
        Args::Up => M::up(&connection)
            .await
            .context("failed to create tables")?,
        Args::Down => M::down(&connection)
            .await
            .context("failed to drop tables")?,
    }

    Ok(())
}
