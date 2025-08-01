mod database;
mod resource;

use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Error, Result, anyhow};
use clap::{Parser, Subcommand};
use libsql::{Builder, Connection};
use log::info;
use pollster::FutureExt;

use crate::database::initialse_database;
use crate::resource::Cli as ResourceCli;

#[derive(Clone, Debug)]
enum DBArg {
    Memory,
    Path(PathBuf),
}

impl FromStr for DBArg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let db_arg = match s {
            ":memory:" => Self::Memory,
            path => DBArg::Path(PathBuf::from(path)),
        };

        Ok(db_arg)
    }
}

impl DBArg {
    fn build_connection(self) -> Result<Connection> {
        match self {
            DBArg::Memory => Builder::new_local(":memory:"),
            DBArg::Path(path_buf) => Builder::new_local(path_buf),
        }
        .build()
        .block_on()?
        .connect()
        .map_err(|e| anyhow!(e))
    }
}

/// Manage resource bookings
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Database to connect to, either a path or ':memory:'
    database: DBArg,
    /// Subcommand to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    InitDatabase,
    Resource(ResourceCli),
}

impl Commands {
    fn dispatch(self, database_connection: Connection) -> Result<()> {
        info!("Dispatching to {:?}", self);
        match self {
            Commands::InitDatabase => initialse_database(database_connection)?,
            Commands::Resource(cli) => cli.run(database_connection)?,
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    env_logger::init();

    info!("Starting booking manager...");

    let cli = Cli::parse();
    info!("Provided arguments: {:?}", cli);

    let Cli { database, command } = cli;

    info!("Connecting to database...");
    let database_connection = database.build_connection()?;
    info!("Database connection made.");

    command.dispatch(database_connection)?;

    info!("Booking manager finished.");
    Ok(())
}
