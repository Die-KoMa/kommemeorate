// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::{Path, PathBuf};
use std::{fmt::Debug, fs::read_to_string};

use anyhow::{Context, Error, Result};
use config::{Config, Environment, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Configuration {
    telegram: TelegramConfiguration,
    matrix: MatrixConfiguration,
    storage: StorageConfiguration,
    database: DatabaseConfiguration,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelegramConfiguration {
    api_id_file: PathBuf,
    api_hash_file: PathBuf,
    password_file: PathBuf,
    groups: Vec<Group>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Group {
    pub(crate) id: i64,
    pub(crate) name: String,
}

pub(crate) struct Telegram {
    api_id: i32,
    api_hash: String,
    bot_password: String,
    groups: Vec<Group>,
}

impl Telegram {
    pub(crate) fn api_credentials(&self) -> (i32, &str) {
        (self.api_id, &self.api_hash)
    }

    pub(crate) fn bot_token(&self) -> &str {
        &self.bot_password
    }

    pub(crate) fn groups(&self) -> impl Iterator<Item = &Group> {
        self.groups.iter()
    }
}

impl Debug for Telegram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Telegram")
            .field("api_id", &"[REDACTED]")
            .field("api_hash", &"[REDACTED]")
            .field("bot_password", &"[REDACTED]")
            .field("groups", &self.groups)
            .finish()
    }
}

impl TryFrom<&TelegramConfiguration> for Telegram {
    type Error = Error;

    fn try_from(value: &TelegramConfiguration) -> std::result::Result<Self, Self::Error> {
        let api_id = read_to_string(value.api_id_file.clone())?.parse()?;
        let api_hash = read_to_string(value.api_hash_file.clone())?;
        let bot_password = read_to_string(value.password_file.clone())?;

        Ok(Self {
            api_id,
            api_hash,
            bot_password,
            groups: value.groups.clone(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatrixConfiguration {
    homeserver: String,
    username: String,
    password_file: PathBuf,
    rooms: Vec<Room>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Room {
    pub(crate) name: String,
    pub(crate) address: String,
}

pub(crate) struct Matrix {
    homeserver: String,
    username: String,
    password: String,
    rooms: Vec<Room>,
}

#[allow(unused)]
impl Matrix {
    pub(crate) fn homeserver(&self) -> &str {
        &self.homeserver
    }

    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn password(&self) -> &str {
        &self.password
    }

    pub(crate) fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.iter()
    }
}

impl Debug for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matrix")
            .field("homeserver", &self.homeserver)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("rooms", &self.rooms)
            .finish()
    }
}

impl TryFrom<&MatrixConfiguration> for Matrix {
    type Error = Error;

    fn try_from(value: &MatrixConfiguration) -> std::result::Result<Self, Self::Error> {
        let password = read_to_string(value.password_file.clone())?;

        Ok(Self {
            homeserver: value.homeserver.clone(),
            username: value.username.clone(),
            rooms: value.rooms.clone(),
            password,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageConfiguration {
    path: PathBuf,
}

impl StorageConfiguration {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseConfiguration {
    url: String,
}

impl DatabaseConfiguration {
    pub(crate) fn url(&self) -> &str {
        &self.url
    }
}

impl Configuration {
    pub(crate) fn load(config_file: PathBuf) -> Result<Self> {
        let settings = Config::builder()
            .add_source(config::File::new(
                config_file
                    .to_str()
                    .context("invalid configuration file path {config_file:?}")?,
                FileFormat::Toml,
            ))
            .add_source(Environment::with_prefix("KOMMEMEORATE"))
            .build()?;

        settings
            .try_deserialize::<Configuration>()
            .context("failed to parse configuration")
    }

    pub(crate) fn telegram(&self) -> Result<Telegram> {
        (&self.telegram).try_into()
    }

    #[allow(unused)]
    pub(crate) fn matrix(&self) -> Result<Matrix> {
        (&self.matrix).try_into()
    }

    pub(crate) fn database(&self) -> &DatabaseConfiguration {
        &self.database
    }

    pub(crate) fn storage(&self) -> &StorageConfiguration {
        &self.storage
    }
}

#[cfg(test)]
mod test {
    use test_log::test;

    use crate::config::{Matrix, Telegram};

    const NEEDLE: &str = "0x23acab";
    const REDACTED: &str = "[REDACTED]";

    #[test]
    fn telegram_debug() {
        let telegram = Telegram {
            api_id: 0,
            api_hash: String::new(),
            bot_password: NEEDLE.to_string(),
            groups: vec![],
        };

        assert!(format!("{telegram:?}").contains(REDACTED));
        assert!(!format!("{telegram:?}").contains(NEEDLE));
    }

    #[test]
    fn matrix_debug() {
        let matrix = Matrix {
            homeserver: String::new(),
            username: String::new(),
            password: NEEDLE.to_string(),
            rooms: vec![],
        };

        assert!(format!("{matrix:?}").contains(REDACTED));
        assert!(!format!("{matrix:?}").contains(NEEDLE));
    }
}
