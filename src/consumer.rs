// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use std::fmt::Debug;

use chrono::{DateTime, Utc};
use grammers_client::types::Chat;

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
