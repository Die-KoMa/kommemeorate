// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

mod cli;
mod service;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use env_logger::Env;
use service::{Notifications, ReloadSignals, ShutdownSignals};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    Notifications::starting()?;
    let _args = Cli::parse();
    let mut reload_signals = ReloadSignals::new()?;
    let mut shutdown_signals = ShutdownSignals::new()?;
    log::info!("running");
    Notifications::ready()?;

    loop {
        tokio::select! {
            _ = reload_signals.reload() => {
                Notifications::reloading()?;
                log::info!("reloading");
                Notifications::ready()?;
            }
            _ = shutdown_signals.shutdown() => {
                Notifications::stopping()?;
                log::info!("shutting down");
                break;
            }
        }
    }

    Ok(())
}
