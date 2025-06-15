// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use std::io::Error;

use sd_notify::{NotifyState, notify};
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

#[derive(Debug)]
pub(crate) struct Notifications {}

impl Notifications {
    pub(crate) fn starting() -> Result<(), Error> {
        notify(false, &[NotifyState::Status("starting up")])
    }

    pub(crate) fn ready() -> Result<(), Error> {
        notify(
            false,
            &[
                NotifyState::Ready,
                NotifyState::Status("ready to process memes"),
            ],
        )
    }

    pub(crate) fn reloading() -> Result<(), Error> {
        notify(
            false,
            &[
                NotifyState::Reloading,
                NotifyState::monotonic_usec_now()?,
                NotifyState::Status("reloading configuration"),
            ],
        )
    }

    pub(crate) fn stopping() -> Result<(), Error> {
        notify(
            false,
            &[NotifyState::Stopping, NotifyState::Status("shutting down")],
        )
    }

    #[allow(unused)]
    pub(crate) fn failed(code: u32, message: &str) -> Result<(), Error> {
        notify(
            false,
            &[NotifyState::Status(message), NotifyState::Errno(code)],
        )
    }
}
