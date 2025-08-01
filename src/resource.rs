use anyhow::{Result, anyhow};
use clap::{Args, Subcommand};
use libsql::Connection;
use log::{debug, error, info};
use pollster::FutureExt;

#[derive(Debug, Args)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

const SQL_ERROR_UNIQUE: i32 = 2067;

pub fn create_resource(connection: Connection, name: String) -> Result<i64> {
    if name.chars().all(|c| c.is_ascii_digit()) {
        return Err(anyhow!("All characters of '{}' are digits.", name));
    }

    connection
        .execute("INSERT INTO resource (name) VALUES (?1)", [name.clone()])
        // keep it synchronous to keep it simple
        .block_on()
        .map_err(|e| match e {
            libsql::Error::SqliteFailure(SQL_ERROR_UNIQUE, ref err_str) => {
                info!("Name '{}' already exists as a resource", name);
                debug!("sqlite error message: {}", err_str);
                return anyhow!("Name '{}' already exists as a resource", name);
            }
            _ => {
                error!(
                    "Unexpected error occured while inserting resource '{}', {}",
                    name, e
                );
                anyhow!(
                    "Unexpected error occured while inserting resource '{}', {}",
                    name,
                    e
                )
            }
        })
        .map(|num_inserted| debug!("inserted {} values", num_inserted))?;

    let inserted_id = connection.last_insert_rowid();
    info!("resource '{}' inserted with id '{}'", name, inserted_id);
    Ok(inserted_id)
}

pub fn get_resource_id(connection: Connection, id: u32) -> Result<String> {
    info!("querying database for resource id {}", id);
    let binding = connection
        .query("SELECT name FROM resource WHERE id = ?1", [id])
        .block_on()
        .map_err(|e| anyhow!("error occured while getting id '{}': {}", id, e))?
        .next()
        .block_on()?
        .ok_or(anyhow!("no results found"))?;
    let found_str = binding
        .get_str(0)
        .expect("result column should be a TEXT field");
    Ok(found_str.to_string())
}

pub fn get_resource_name(connection: Connection, name: String) -> Result<i64> {
    info!("querying database for resource name {}", name);
    let found_id = connection
        .query("SELECT id FROM resource WHERE name = ?1", [name.clone()])
        .block_on()
        .map_err(|e| anyhow!("error occured while getting name '{}': {}", name, e))?
        .next()
        .block_on()?
        .ok_or(anyhow!("no results found"))?
        .get(0)?;
    Ok(found_id)
}

impl Cli {
    pub fn run(self, connection: Connection) -> Result<()> {
        match self.command {
            Commands::Create { name } => {
                let inserted_id = create_resource(connection, name)?;
                println!("{}", inserted_id);
                Ok(())
            }
            Commands::Search { name } => {
                let found_id = get_resource_name(connection, name)?;
                println!("{}", found_id);
                Ok(())
            }
            Commands::Get { id } => {
                let found_name = get_resource_id(connection, id)?;
                println!("{}", found_name);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new resource
    #[command(arg_required_else_help = true)]
    Create { name: String },
    /// Search for resources
    #[command(arg_required_else_help = true)]
    Search { name: String },
    /// Get the name of a resource
    #[command(arg_required_else_help = true)]
    Get { id: u32 },
}
