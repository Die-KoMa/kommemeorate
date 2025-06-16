// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

mod db;

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::{
    PgConnection,
    dsl::{delete, insert_into, update},
};
use grammers_client::types::Chat;
use tokio::{
    fs, select,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::config::{DatabaseConfiguration, StorageConfiguration};

#[derive(Debug)]
pub(crate) enum Source {
    Telegram {
        account: Option<String>,
        channel: Option<String>,
        id: i32,
    },
    #[allow(unused)]
    Matrix { account: String, channel: String },
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
    data: Vec<u8>,
    spoiler: bool,
    text: String,
    timestamp: NaiveDateTime,
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
        timestamp: NaiveDateTime,
    ) -> Self {
        Self {
            data,
            spoiler,
            text,
            timestamp,
        }
    }
}

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

fn file_name(source: &Source) -> String {
    match source {
        Source::Telegram {
            account,
            channel,
            id: message_id,
        } => format!(
            "telegram-{}-{}-{message_id}.jpg",
            channel.clone().unwrap_or_default(),
            account.clone().unwrap_or_default()
        ),
        Source::Matrix { account, channel } => {
            format!("matrix-{channel}-{account}")
        }
    }
}

async fn save_meme(
    path: PathBuf,
    db: &mut PgConnection,
    image: MemeImage,
    source: Source,
) -> Result<()> {
    use db::{models::NewMeme, schema::memes};
    use diesel::prelude::*;
    log::debug!("saving meme: {source:?}");

    let file = file_name(&source);
    match source {
        Source::Telegram {
            account,
            channel,
            id: message_id,
        } => {
            let mut file_path = path.clone();
            file_path.push(file.clone());
            log::debug!("writing to {file_path:?}");
            fs::write(file_path, &image.data).await?;

            let new_meme = NewMeme {
                spoiler: image.spoiler,
                text: &image.text,
                timestamp: image.timestamp,
                account: &account.unwrap_or_default(),
                channel: &channel.unwrap_or_default(),
                filename: &file,
                telegram_id: Some(message_id),
            };
            let result = insert_into(memes::table).values(&new_meme).execute(db);
            log::debug!("inserted meme: {result:#?}");
        }
        _ => todo!("Matrix is not yet supported"),
    }

    Ok(())
}

async fn update_meme(
    path: PathBuf,
    db: &mut PgConnection,
    image: MemeImage,
    source: Source,
) -> Result<()> {
    use db::schema::memes::dsl::{
        account, channel, filename, memes, spoiler, telegram_id, text, timestamp,
    };
    use diesel::prelude::*;
    log::debug!("updating meme: {source:?}");

    let file = file_name(&source);
    match source {
        Source::Telegram {
            account: message_account,
            channel: message_channel,
            id: message_id,
        } => {
            let mut file_path = path.clone();
            file_path.push(file.clone());
            fs::write(file_path, &image.data).await?;

            update(memes.filter(telegram_id.eq(Some(message_id))))
                .set((
                    spoiler.eq(image.spoiler),
                    text.eq(image.text),
                    timestamp.eq(image.timestamp),
                    account.eq(message_account.unwrap_or_default()),
                    channel.eq(message_channel.unwrap_or_default()),
                    filename.eq(file),
                ))
                .execute(db)?;
        }
        _ => todo!("Matrix is not yet supported"),
    }

    Ok(())
}

async fn delete_meme(path: PathBuf, db: &mut PgConnection, source: Source) -> Result<()> {
    use db::schema::memes::dsl::{filename, id, memes, telegram_id};
    use diesel::prelude::*;

    log::debug!("deleting meme: {source:?}");

    match source {
        Source::Telegram {
            account: _,
            channel: _,
            id: message_id,
        } => {
            let files = memes
                .select((id, filename))
                .filter(telegram_id.eq(Some(message_id)))
                .load::<(i32, String)>(db)?;

            for (meme_id, file) in files {
                let mut file_path = path.clone();
                file_path.push(Path::new(&file));
                fs::remove_file(file_path).await?;
                delete(memes.find(meme_id)).execute(db)?;
            }
        }
        _ => todo!("Matrix is not yet supported"),
    }

    Ok(())
}

async fn process(
    storage: StorageConfiguration,
    database: DatabaseConfiguration,
    mut control: Receiver<Command>,
    mut consumer: Receiver<MemeEvent>,
) -> TaskResult {
    log::info!("starting storage");

    let path = storage.path().to_path_buf();
    let mut db = db::connect(database.url())?;
    log::debug!("connected to database");

    async fn handle_event(path: PathBuf, db: &mut PgConnection, event: MemeEvent) -> Result<()> {
        log::info!("new event: {event:#?}");
        match event {
            MemeEvent::New { image, source } => save_meme(path, db, image, source).await?,
            MemeEvent::Updated { image, source } => update_meme(path, db, image, source).await?,
            MemeEvent::Deleted { source } => delete_meme(path, db, source).await?,
        };

        Ok(())
    }

    loop {
        select! {
            Some(event) = consumer.recv() => {
                handle_event(path.clone(), &mut db, event).await?;
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
