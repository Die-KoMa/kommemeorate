// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use anyhow::{Context, Error, Result};

use matrix_sdk::{Client, LoopCtrl, config::SyncSettings, reqwest::Url};
use tokio::{
    select,
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    task::JoinHandle,
};

use crate::{
    config,
    consumer::{MemeEvent, MemeImage, Source},
};

#[derive(Debug)]
pub struct Matrix {
    task: JoinHandle<Result<Sender<MemeEvent>, Error>>,
    control: Sender<Command>,
}

impl Matrix {
    pub(crate) fn new(config: config::Matrix, consumer: Sender<MemeEvent>) -> Result<Self> {
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

    pub(crate) async fn reload(self, config: config::Matrix) -> Result<Self> {
        log::info!("restarting matrix bot");
        if !self.control.is_closed() {
            self.control.send(Command::Shutdown).await?;
        }
        let consumer = self.task.await??;
        Self::new(config, consumer)
    }

    pub(crate) async fn shutdown(self) -> Result<()> {
        log::info!("shutting down matrix bot");
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

type RoomMap = HashMap<String, String>;

async fn process(
    config: config::Matrix,
    mut control: Receiver<Command>,
    consumer: Sender<MemeEvent>,
) -> Result<Sender<MemeEvent>> {
    log::info!("starting matrix bot");

    let url = Url::parse(config.homeserver()).context("failed to parse homeserver URL")?;
    let client = Client::new(url).await?;
    log::debug!("connected to homeserver");
    let result = client
        .matrix_auth()
        .login_username(config.username(), config.password())
        .initial_device_display_name("kommemeorate")
        .await?;
    log::debug!("{result:#?}");

    log::debug!("connected to matrix");
    let mut rooms: RoomMap = HashMap::from_iter(
        config
            .rooms()
            .map(|room| (room.address.clone(), room.name.clone())),
    );

    fn is_relevant(rooms: &mut RoomMap, room: String) -> bool {
        rooms.contains_key(&room)
    }

    async fn handle_message(
        client: &Client,
        groups: &mut RoomMap,
        consumer: Sender<MemeEvent>,
        //message: update::Message,
        is_edit: bool,
    ) -> Result<()> {
        // if is_relevant(groups, message.chat()) {
        //     if let Some(Media::Photo(photo)) = message.media() {
        //         // don't collect disappearing photos
        //         if photo.ttl_seconds().is_none() {
        //             let spoiler = photo.is_spoiler();
        //             let mut bytes = Vec::new();
        //             let mut download = client.iter_download(&message.media().expect("is present"));

        //             while let Some(chunk) = download.next().await? {
        //                 bytes.extend(chunk);
        //             }

        //             let timestamp = if is_edit {
        //                 message.edit_date().expect("is edited")
        //             } else {
        //                 message.date()
        //             };
        //             let image = MemeImage::new(
        //                 bytes,
        //                 spoiler,
        //                 message.text().to_string(),
        //                 timestamp.naive_utc(),
        //             );
        //             let source = Source::telegram(
        //                 message.sender(),
        //                 groups
        //                     .get(&message.chat().id())
        //                     .map(|group| group.name.as_str()),
        //                 message.id(),
        //             );

        //             let event = if is_edit {
        //                 MemeEvent::edit(image, source)
        //             } else {
        //                 MemeEvent::new(image, source)
        //             };

        //             return Ok(consumer.send(event).await?);
        //         }
        //     }
        // }
        Ok(())
    }

    async fn handle_delete(
        consumer: Sender<MemeEvent>,
        //        message: update::MessageDeletion,
    ) -> Result<()> {
        // assert!(message.channel_id().is_none());

        // for &id in message.messages() {
        //     consumer
        //         .send(MemeEvent::delete(Source::telegram(None, None, id)))
        //         .await?;
        // }

        Ok(())
    }

    let (tx, rx) = oneshot::channel::<()>();

    loop {
        select! {
            update = client.sync_with_callback(SyncSettings::default(), async |_| { if rx.is_empty() { LoopCtrl::Continue } else { LoopCtrl::Break }}) => {
                log::debug!("update: {update:#?}");
            }
            // update = client.next_update() => {
            //     match update {
            //         Ok(Update::NewMessage(message))  => {
            //             handle_message(&client, &mut groups, consumer.clone(), message, false).await?
            //         }
            //         Ok(Update::MessageEdited(message)) => {
            //             handle_message(&client, &mut groups, consumer.clone(), message, true).await?
            //         }
            //         Ok(Update::MessageDeleted(message)) => {
            //             handle_delete(consumer.clone(), message).await?
            //         },
            //         Err(err) => {
            //             log::error!("error: {err:?}");
            //         }
            //         _ => {
            //         }
            //     }
            // }

            Some(command) = control.recv() => {
                match command {
                    Command::Shutdown => {
                        let _ = tx.send(());
                        break
                    },
                }
            }
        }
    }

    client.logout().await?;
    drop(client);

    Ok(consumer)
}
