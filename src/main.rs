// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

mod cli;
mod config;
mod service;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use config::Configuration;
use env_logger::Env;
use service::{Notifications, ReloadSignals, ShutdownSignals};

async fn process() -> Result<()> {
    Notifications::starting()?;
    let args = Cli::parse();
    let configuration = Configuration::load(args.config)?;
    let mut reload_signals = ReloadSignals::new()?;
    let mut shutdown_signals = ShutdownSignals::new()?;
    log::info!("running");
    Notifications::ready()?;

    log::debug!("{:#?}", configuration.telegram()?);

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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    match process().await {
        Ok(_) => {}
        Err(err) => {
            _ = Notifications::failed(1312, &err.to_string());
            return Err(err);
        }
    }

    Ok(())
}
