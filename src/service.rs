// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use std::io::Error;

use tokio::{
    select,
    signal::unix::{Signal, SignalKind, signal},
};

#[derive(Debug)]
pub(crate) struct ReloadSignals {
    hangup: Signal,
}

impl ReloadSignals {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            hangup: signal(SignalKind::hangup())?,
        })
    }

    pub(crate) async fn reload(&mut self) -> Option<()> {
        self.hangup.recv().await
    }
}

#[derive(Debug)]
pub(crate) struct ShutdownSignals {
    interrupt: Signal,
    terminate: Signal,
    quit: Signal,
}

impl ShutdownSignals {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            interrupt: signal(SignalKind::interrupt())?,
            terminate: signal(SignalKind::terminate())?,
            quit: signal(SignalKind::quit())?,
        })
    }

    pub(crate) async fn shutdown(&mut self) -> Option<()> {
        select! {
            result = self.interrupt.recv() => result,
            result = self.terminate.recv() => result,
            result = self.quit.recv() => result,
        }
    }
}
