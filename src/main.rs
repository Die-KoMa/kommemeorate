// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

mod cli;
mod config;
mod consumer;
mod matrix;
mod service;
mod telegram;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use config::Configuration;
use consumer::Consumer;
use env_logger::Env;
#[allow(unused)]
use matrix::Matrix;
use service::{Notifications, ReloadSignals, ShutdownSignals};
use telegram::Telegram;

async fn process() -> Result<()> {
    Notifications::starting()?;
    let args = Cli::parse();
    let mut configuration = Configuration::load(args.config.clone())?;
    let mut reload_signals = ReloadSignals::new()?;
    let mut shutdown_signals = ShutdownSignals::new()?;
    let (mut consumer, meme_consumer) = Consumer::new(
        configuration.storage().clone(),
        configuration.database().clone(),
    )?;
    let mut telegram = Telegram::new(configuration.telegram()?, meme_consumer.clone())?;
    //let mut matrix = Matrix::new(configuration.matrix()?, meme_consumer)?;
    log::info!("running");
    Notifications::ready()?;

    loop {
        tokio::select! {
            _ = reload_signals.reload() => {
                Notifications::reloading()?;
                log::info!("reloading");
                configuration = Configuration::load(args.config.clone())?;
                consumer = consumer.reload(configuration.storage().clone(), configuration.database().clone()).await?;
                telegram = telegram.reload(configuration.telegram()?).await?;
                //matrix = matrix.reload(configuration.matrix()?).await?;
                Notifications::ready()?;
            }
            _ = shutdown_signals.shutdown() => {
                Notifications::stopping()?;
                log::info!("shutting down");
                //matrix.shutdown().await?;
                telegram.shutdown().await?;
                consumer.shutdown().await?;
                break;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("initialising logging");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    eprintln!("initialised logging");
    log::info!("starting kommemeorate");

    match process().await {
        Ok(_) => {}
        Err(err) => {
            _ = Notifications::failed(1312, &err.to_string());
            return Err(err);
        }
    }

    Ok(())
}
