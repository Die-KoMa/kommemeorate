// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

mod cli;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let _args = Cli::parse();

    Ok(())
}
