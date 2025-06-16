// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

pub(super) mod models;
pub(super) mod schema;

use anyhow::{Result, anyhow};
use diesel::{Connection, PgConnection, pg::Pg};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::error::Error;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

pub(crate) fn connect(url: &str) -> Result<PgConnection> {
    let mut connection = PgConnection::establish(url)?;

    run_migrations(&mut connection).map_err(|err| anyhow!(err.to_string()))?;

    Ok(connection)
}
