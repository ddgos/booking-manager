use anyhow::{Context, Result};
use libsql::Connection;
use log::{debug, error};
use pollster::FutureExt;

const CREATE_STATEMENTS: [&str; 2] = [
    include_str!("./sql/create/resource.sql"),
    include_str!("./sql/create/booking.sql"),
];

pub fn initialse_database(db_connection: Connection) -> Result<()> {
    {
        for sql in CREATE_STATEMENTS {
            debug!("Running SQL {}", sql);
            db_connection
                .execute(sql, ())
                .block_on()
                .inspect_err(|e| error!("Error {} occured while running\n{}", e, sql))
                .with_context(|| format!("while running\n{}", sql))?;
        }
        Ok(())
    }
}
