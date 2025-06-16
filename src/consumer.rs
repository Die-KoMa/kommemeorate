// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use std::fmt::Debug;

use anyhow::Result;
use chrono::{DateTime, Utc};
use grammers_client::types::Chat;
use tokio::{
    select,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::config::{DatabaseConfiguration, StorageConfiguration};

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Source {
    Telegram {
        account: Option<String>,
        channel: Option<String>,
        id: i32,
    },
    Matrix {
        account: String,
        channel: String,
    },
}

impl Source {
    pub(crate) fn telegram(chat: Option<Chat>, channel: Option<&str>, id: i32) -> Self {
        Self::Telegram {
            account: match chat {
                Some(Chat::User(user)) => user.username().map(|name| name.to_string()),
                Some(Chat::Group(group)) => group.title().map(|name| name.to_string()),
                Some(Chat::Channel(channel)) => Some(channel.title().to_string()),
                None => None,
            },
            channel: channel.map(|name| name.to_string()),
            id,
        }
    }
}

pub(crate) struct MemeImage {
    #[allow(unused)]
    data: Vec<u8>,
    spoiler: bool,
    text: String,
    timestamp: DateTime<Utc>,
}

impl Debug for MemeImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemeImage")
            .field("data", &"[elided]")
            .field("spoiler", &self.spoiler)
            .field("text", &self.text)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

impl MemeImage {
    pub(crate) fn new(
        data: Vec<u8>,
        spoiler: bool,
        text: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            data,
            spoiler,
            text,
            timestamp,
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum MemeEvent {
    New { image: MemeImage, source: Source },
    Updated { image: MemeImage, source: Source },
    Deleted { source: Source },
}

impl MemeEvent {
    pub(crate) fn new(image: MemeImage, source: Source) -> Self {
        Self::New { image, source }
    }

    pub(crate) fn edit(image: MemeImage, source: Source) -> Self {
        Self::Updated { image, source }
    }

    pub(crate) fn delete(source: Source) -> Self {
        Self::Deleted { source }
    }
}

#[derive(Debug)]
pub(crate) enum Command {
    Shutdown,
}

type TaskResult = Result<(Receiver<Command>, Receiver<MemeEvent>)>;

#[derive(Debug)]
pub(crate) struct Consumer {
    task: JoinHandle<TaskResult>,
    control: Sender<Command>,
}

impl Consumer {
    pub(crate) fn new(
        storage: StorageConfiguration,
        database: DatabaseConfiguration,
    ) -> Result<(Self, Sender<MemeEvent>)> {
        let (control, rx) = mpsc::channel(8);
        let (tx, consumer) = mpsc::channel(32);

        Ok((
            Self::with_control_and_consumer(storage, database, control, rx, consumer)?,
            tx,
        ))
    }

    fn with_control_and_consumer(
        storage: StorageConfiguration,
        database: DatabaseConfiguration,
        control: Sender<Command>,
        rx: Receiver<Command>,
        consumer: Receiver<MemeEvent>,
    ) -> Result<Self> {
        let task = tokio::spawn(process(storage, database, rx, consumer));

        Ok(Self { task, control })
    }

    pub(crate) async fn reload(
        self,
        storage: StorageConfiguration,
        database: DatabaseConfiguration,
    ) -> Result<Self> {
        log::info!("restarting storage");
        let control = self.control.clone();
        self.control.send(Command::Shutdown).await?;
        let (rx, consumer) = self.task.await??;
        Self::with_control_and_consumer(storage, database, control, rx, consumer)
    }

    pub(crate) async fn shutdown(self) -> Result<()> {
        log::info!("shutting down storage");
        self.control.send(Command::Shutdown).await?;
        self.task.await??;
        Ok(())
    }
}

#[allow(unused)]
async fn process(
    storage: StorageConfiguration,
    database: DatabaseConfiguration,
    mut control: Receiver<Command>,
    mut consumer: Receiver<MemeEvent>,
) -> TaskResult {
    log::info!("starting storage");

    async fn handle_event(event: MemeEvent) -> Result<()> {
        log::info!("new event: {event:#?}");
        match event {
            MemeEvent::New { image, source } => (),
            MemeEvent::Updated { image, source } => (),
            MemeEvent::Deleted { source } => (),
        };

        Ok(())
    }

    loop {
        select! {
            Some(event) = consumer.recv() => {
                handle_event(event).await?;
            }

            Some(command) = control.recv() => {
                match command {
                    Command::Shutdown => break
                }
            }
        }
    }

    Ok((control, consumer))
}
