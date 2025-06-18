// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use anyhow::{Error, Result};
use grammers_client::{
    Client, Config, InitParams,
    session::Session,
    types::{Chat, Media, Update, update},
};
use tokio::{
    select,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    config,
    consumer::{MemeEvent, MemeImage, Source},
};

#[derive(Debug)]
pub struct Telegram {
    task: JoinHandle<Result<Sender<MemeEvent>, Error>>,
    control: Sender<Command>,
}

impl Telegram {
    pub(crate) fn new(config: config::Telegram, consumer: Sender<MemeEvent>) -> Result<Self> {
        let (tx, rx) = mpsc::channel(8);
        let task = tokio::spawn(async move {
            let result = process(config, rx, consumer).await;
            if let Err(ref err) = result {
                log::error!("{err}");
            }
            result
        });

        Ok(Self { task, control: tx })
    }

    pub(crate) async fn reload(self, config: config::Telegram) -> Result<Self> {
        log::info!("restarting telegram bot");
        if !self.control.is_closed() {
            self.control.send(Command::Shutdown).await?;
        }
        let consumer = self.task.await??;
        Self::new(config, consumer)
    }

    pub(crate) async fn shutdown(self) -> Result<()> {
        log::info!("shutting down telegram bot");
        if !self.control.is_closed() {
            self.control.send(Command::Shutdown).await?;
        }
        self.task.await??;
        Ok(())
    }
}

#[derive(Debug)]
enum Command {
    Shutdown,
}

#[derive(Debug)]
struct Group {
    name: String,
    chat: Option<Chat>,
}

impl From<config::Group> for Group {
    fn from(value: config::Group) -> Self {
        Self {
            name: value.name,
            chat: None,
        }
    }
}

type GroupMap = HashMap<i64, Group>;

async fn process(
    config: config::Telegram,
    mut control: Receiver<Command>,
    consumer: Sender<MemeEvent>,
) -> Result<Sender<MemeEvent>> {
    log::info!("starting telegram bot");

    let (api_id, api_hash) = config.api_credentials();

    let client = Client::connect(Config {
        session: Session::new(),
        api_id,
        api_hash: api_hash.to_string(),
        params: Default::default(),
    })
    .await?;

    log::debug!("connected to telegram");
    let _bot = client.bot_sign_in(config.bot_token()).await?;
    let mut groups: GroupMap = HashMap::from_iter(
        config
            .groups()
            .map(|group| (group.id, Group::from(group.clone()))),
    );

    fn is_relevant(groups: &mut GroupMap, chat: Chat) -> bool {
        if let Some(group) = groups.get_mut(&chat.id()) {
            group.chat = Some(chat);
            return true;
        }

        false
    }

    async fn handle_message(
        client: &Client,
        groups: &mut GroupMap,
        consumer: Sender<MemeEvent>,
        message: update::Message,
        is_edit: bool,
    ) -> Result<()> {
        if is_relevant(groups, message.chat()) {
            if let Some(Media::Photo(photo)) = message.media() {
                // don't collect disappearing photos
                if photo.ttl_seconds().is_none() {
                    let spoiler = photo.is_spoiler();
                    let mut bytes = Vec::new();
                    let mut download = client.iter_download(&message.media().expect("is present"));

                    while let Some(chunk) = download.next().await? {
                        bytes.extend(chunk);
                    }

                    let timestamp = if is_edit {
                        message.edit_date().expect("is edited")
                    } else {
                        message.date()
                    };
                    let image = MemeImage::new(
                        bytes,
                        spoiler,
                        message.text().to_string(),
                        timestamp.naive_utc(),
                    );
                    let source = Source::telegram(
                        message.sender(),
                        groups
                            .get(&message.chat().id())
                            .map(|group| group.name.as_str()),
                        message.id(),
                    );

                    let event = if is_edit {
                        MemeEvent::edit(image, source)
                    } else {
                        MemeEvent::new(image, source)
                    };

                    return Ok(consumer.send(event).await?);
                }
            }
        }
        Ok(())
    }

    async fn handle_delete(
        consumer: Sender<MemeEvent>,
        message: update::MessageDeletion,
    ) -> Result<()> {
        assert!(message.channel_id().is_none());

        for &id in message.messages() {
            consumer
                .send(MemeEvent::delete(Source::telegram(None, None, id)))
                .await?;
        }

        Ok(())
    }

    loop {
        select! {
            update = client.next_update() => {
                match update {
                    Ok(Update::NewMessage(message))  => {
                        handle_message(&client, &mut groups, consumer.clone(), message, false).await?
                    }
                    Ok(Update::MessageEdited(message)) => {
                        handle_message(&client, &mut groups, consumer.clone(), message, true).await?
                    }
                    Ok(Update::MessageDeleted(message)) => {
                        handle_delete(consumer.clone(), message).await?
                    },
                    Err(err) => {
                        log::error!("error: {err:?}");
                    }
                    _ => {
                    }
                }
            }

            Some(command) = control.recv() => {
                match command {
                    Command::Shutdown => break,
                }
            }
        }
    }

    client.sign_out().await?;
    drop(client);

    Ok(consumer)
}
