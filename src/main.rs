// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

mod cli;
mod config;
mod consumer;
mod service;
mod telegram;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use config::Configuration;
use env_logger::Env;
use service::{Notifications, ReloadSignals, ShutdownSignals};
use telegram::Telegram;
use tokio::sync::mpsc;

async fn process() -> Result<()> {
    Notifications::starting()?;
    let args = Cli::parse();
    let mut configuration = Configuration::load(args.config.clone())?;
    let mut reload_signals = ReloadSignals::new()?;
    let mut shutdown_signals = ShutdownSignals::new()?;
    let (consumer, mut rx) = mpsc::channel(32);
    let mut telegram = Telegram::new(configuration.telegram()?, consumer)?;
    log::info!("running");
    Notifications::ready()?;

    loop {
        tokio::select! {
            _ = reload_signals.reload() => {
                Notifications::reloading()?;
                log::info!("reloading");
                configuration = Configuration::load(args.config.clone())?;
                telegram = telegram.reload(configuration.telegram()?).await?;
                Notifications::ready()?;
            }
            _ = shutdown_signals.shutdown() => {
                Notifications::stopping()?;
                log::info!("shutting down");
                telegram.shutdown().await?;
                break;
            }
            meme = rx.recv() => {
                log::info!("new meme: {meme:#?}");
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
